use crate::clock::Clock;
use crate::data::{MultiWriter, OutputFormat, SessionHeader};
use crate::error::Error;
use crate::renderer::RenderCommand;
use mlua::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::info;

pub mod api_clock;
pub mod api_data;
pub mod api_rand;
pub mod api_stim;
pub mod api_trial;

/// Stdlib source embedded at compile time so it is always in sync with the binary.
const STDLIB: &str = include_str!("stdlib.lua");

#[derive(Clone)]
pub(crate) struct HostState {
    pub clock: Clock,
    pub data_store: Arc<Mutex<Option<MultiWriter>>>,
    #[allow(dead_code)]
    pub output_dir: std::path::PathBuf,
    #[allow(dead_code)]
    pub header: SessionHeader,
    pub render_handle: Arc<Mutex<Option<crate::renderer::RenderHandle>>>,
    pub screen_size: Arc<Mutex<(f32, f32)>>,
}

impl HostState {
    pub fn set_format(&self, format: &str) -> anyhow::Result<()> {
        let fmt = OutputFormat::from_str(format).ok_or_else(|| {
            anyhow::anyhow!("Unknown format {:?}. Use csv, json, or both.", format)
        })?;
        let mut store = self.data_store.lock().expect("data_store poisoned");
        if let Some(ref mut mw) = *store {
            mw.set_format(&fmt)?;
        }
        Ok(())
    }
}

pub struct ScriptHost {
    lua: Lua,
    state: HostState,
}

impl ScriptHost {
    pub fn new(
        clock: Clock,
        output_dir: &Path,
        header: SessionHeader,
        seed: Option<u64>,
    ) -> Result<Self, Error> {
        let lua = Lua::new();

        lua.sandbox(true).map_err(Error::Script)?;

        std::fs::create_dir_all(output_dir).map_err(Error::Io)?;

        let multi = MultiWriter::new(output_dir, header.clone())
            .map_err(|e| Error::Clock(e.to_string()))?;

        let state = HostState {
            clock,
            data_store: Arc::new(Mutex::new(Some(multi))),
            output_dir: output_dir.to_path_buf(),
            header,
            render_handle: Arc::new(Mutex::new(None)),
            screen_size: Arc::new(Mutex::new((1920.0, 1080.0))),
        };

        let globals = lua.globals();

        api_trial::register(&lua, &state)?;
        api_data::register(&lua, &state)?;

        globals.set("Clock", api_clock::make_clock_table(&lua, &state)?)?;
        globals.set("Rand", api_rand::make_rand_table(&lua, seed)?)?;

        let (sw, sh) = *state.screen_size.lock().expect("screen_size poisoned");
        globals.set("Stim", api_stim::make_stim_table(&lua, sw, sh)?)?;

        globals.set("psychlib_VERSION", env!("CARGO_PKG_VERSION"))?;
        globals.set("_psychlib_screen_h", sh)?;

        drop(globals);

        lua.load(STDLIB)
            .set_name("psychlib_stdlib")
            .exec()
            .map_err(Error::Script)?;

        info!("ScriptHost initialized (Luau sandbox enabled)");

        Ok(Self { lua, state })
    }

    pub fn run(&self, source: &str) -> Result<(), Error> {
        self.lua
            .load(source)
            .set_name("experiment")
            .exec()
            .map_err(Error::Script)
    }

    pub fn run_file(&self, path: &Path) -> Result<(), Error> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("experiment");
        let source = std::fs::read_to_string(path)?;
        self.lua
            .load(&source)
            .set_name(name)
            .exec()
            .map_err(Error::Script)
    }

    #[cfg(test)]
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Attach a renderer and re-register `Stim` with the real window dimensions.
    pub fn attach_renderer(&self, handle: crate::renderer::RenderHandle) {
        if let Some(size) = handle.screen_size() {
            {
                let mut guard = self.state.screen_size.lock().expect("screen_size poisoned");
                *guard = size;
            }

            let (sw, sh) = size;
            if let Ok(stim_tbl) = api_stim::make_stim_table(&self.lua, sw, sh) {
                let _ = self.lua.globals().set("Stim", stim_tbl);
            }
            let _ = self.lua.globals().set("_psychlib_screen_h", sh);
            let _ = self.lua.load(
                "local h = _psychlib_screen_h or 768;                  
                TEXT_SIZE_BODY = math.floor(h * 0.040);
                TEXT_SIZE_FEEDBACK = math.floor(h * 0.052);
                FIX_ARM_LEN = math.floor(h * 0.028);
                FIX_THICKNESS = math.max(2, math.floor(h * 0.004))"
            ).exec();
        }

        *self
            .state
            .render_handle
            .lock()
            .expect("render_handle mutex poisoned") = Some(handle);
    }

    pub fn close(self) -> Result<(), Error> {
        {
            let guard = self
                .state
                .render_handle
                .lock()
                .expect("render_handle mutex poisoned");

            if let Some(handle) = guard.as_ref() {
                let _ = handle.send(RenderCommand::Quit);
            }
        }

        let mut store = self
            .state
            .data_store
            .lock()
            .expect("data store mutex poisoned");

        if let Some(writer) = store.take() {
            writer.close().map_err(|e| Error::Clock(e.to_string()))?;
        }

        info!("ScriptHost closed");
        Ok(())
    }
}
