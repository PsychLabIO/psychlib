use crate::clock::Clock;
use crate::data::SessionHeader;
use crate::renderer::{RenderConfig, RenderLoop};
use crate::script::ScriptHost;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct ExperimentConfig {
    pub participant: String,
    pub script_path: PathBuf,
    pub output_dir: PathBuf,
    pub seed: Option<u64>,
    pub render: RenderConfig,
}

/// Run one experiment session end-to-end.
/// Blocks until the script completes or the window is closed.
/// Must be called from the main thread!
pub fn run(config: ExperimentConfig) -> anyhow::Result<()> {
    let clock = Clock::new();
    let clock_info = clock.info();

    info!("psychlib v{}", env!("CARGO_PKG_VERSION"));
    info!("participant: {}", config.participant);
    info!("script: {}", config.script_path.display());
    info!("output: {}", config.output_dir.display());
    info!(
        "seed: {}",
        config.seed.map_or("entropy".into(), |s| s.to_string())
    );
    info!(
        "platform: {} (hi-res sleep: {})",
        clock_info.platform, clock_info.high_precision_sleep
    );

    let script_stem = config
        .script_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("experiment")
        .to_string();

    let header = SessionHeader::new(
        &config.participant,
        &script_stem,
        config.script_path.to_string_lossy().as_ref(),
        config.seed,
        clock_info,
    );

    std::fs::create_dir_all(&config.output_dir)
        .map_err(|e| anyhow::anyhow!("Cannot create output dir: {e}"))?;

    let (render_handle, event_loop, mut render_loop) =
        RenderLoop::create(config.render.clone(), clock.clone());

    let script_path = config.script_path.clone();
    let output_dir = config.output_dir.clone();
    let seed = config.seed;
    let clock2 = clock.clone();

    let script_thread = std::thread::Builder::new()
        .name("psychlib-script".into())
        .spawn(move || -> anyhow::Result<()> {
            let host = ScriptHost::new(clock2, &output_dir, header, seed)
                .map_err(|e| anyhow::anyhow!("ScriptHost init: {e}"))?;

            host.attach_renderer(render_handle);

            let script_result = host
                .run_file(&script_path)
                .map_err(|e| anyhow::anyhow!("Script error: {e}"));

            let close_result = host
                .close()
                .map_err(|e| anyhow::anyhow!("Data flush error: {e}"));

            script_result.and(close_result)
        })
        .map_err(|e| anyhow::anyhow!("Failed to spawn script thread: {e}"))?;

    event_loop
        .run_app(&mut render_loop)
        .map_err(|e| anyhow::anyhow!("Render loop error: {e}"))?;

    info!("Render loop exited");

    match script_thread.join() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            error!("Script error: {:#}", e);
            return Err(e);
        }
        Err(_panic) => {
            anyhow::bail!("Script thread panicked");
        }
    }

    info!("Session complete");
    Ok(())
}
