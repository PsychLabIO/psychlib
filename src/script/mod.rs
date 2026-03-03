use crate::clock::Clock;
use crate::data::{CsvWriter, DataStore, SessionHeader};
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

#[derive(Clone)]
pub(crate) struct HostState {
    pub clock: Clock,
    pub data_store: Arc<Mutex<Option<Box<dyn DataStore + Send>>>>,
    pub trial_index: Arc<Mutex<usize>>,
    pub block_index: Arc<Mutex<usize>>,
    pub render_handle: Arc<Mutex<Option<crate::renderer::RenderHandle>>>,
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

        let writer =
            CsvWriter::create(output_dir, header).map_err(|e| Error::Clock(e.to_string()))?;

        let state = HostState {
            clock,
            data_store: Arc::new(Mutex::new(Some(
                Box::new(writer) as Box<dyn DataStore + Send>
            ))),
            trial_index: Arc::new(Mutex::new(0)),
            block_index: Arc::new(Mutex::new(0)),
            render_handle: Arc::new(Mutex::new(None)),
        };

        let globals = lua.globals();

        globals.set("Clock", api_clock::make_clock_table(&lua, &state)?)?;
        globals.set("Trial", api_trial::make_trial_table(&lua, &state)?)?;
        globals.set("Data", api_data::make_data_table(&lua, &state)?)?;
        globals.set("Rand", api_rand::make_rand_table(&lua, seed)?)?;
        globals.set("Stim", api_stim::make_stim_table(&lua)?)?;
        globals.set("psychlib_VERSION", env!("CARGO_PKG_VERSION"))?;

        drop(globals);

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

    pub fn attach_renderer(&self, handle: crate::renderer::RenderHandle) {
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
            Box::new(writer)
                .close()
                .map_err(|e| Error::Clock(e.to_string()))?;
        }

        info!("ScriptHost closed");
        Ok(())
    }
}
