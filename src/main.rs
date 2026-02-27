use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{ArgAction, Parser, Subcommand};
use tracing::{debug, error, info, warn};

use psychlib::clock::{Clock, Duration};
use psychlib::renderer::RenderConfig;
use psychlib::runtime::ExperimentConfig;

#[derive(Parser, Debug)]
#[command(
    name = "psychlib",
    version,
    about = "Run psychology experiments written in Luau",
    long_about = "
psychlib runs experiment scripts written in Luau (typed Lua).

Examples:
  psychlib run example.lua --participant P001
  psychlib run example.lua --participant P001 --seed 42 --fullscreen
  psychlib check example.lua
  psychlib info
"
)]
struct Cli {
    #[arg(short, long, global = true, action = ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Run(RunArgs),
    Check(CheckArgs),
    Info,
}

#[derive(Parser, Debug)]
struct RunArgs {
    #[arg(value_name = "SCRIPT")]
    script: PathBuf,

    #[arg(short, long, value_name = "ID", env = "psychlib_PARTICIPANT")]
    participant: String,

    #[arg(
        short,
        long,
        value_name = "DIR",
        default_value = "data",
        env = "psychlib_OUTPUT_DIR"
    )]
    output: PathBuf,

    #[arg(short, long, value_name = "N", env = "psychlib_SEED")]
    seed: Option<u64>,

    #[arg(long)]
    fullscreen: bool,

    #[arg(long, default_value_t = 1024, value_name = "PX")]
    width: u32,

    #[arg(long, default_value_t = 768, value_name = "PX")]
    height: u32,

    #[arg(long)]
    no_benchmark: bool,

    #[arg(long)]
    headless: bool,
}

#[derive(Parser, Debug)]
struct CheckArgs {
    #[arg(value_name = "SCRIPT")]
    script: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = platform::init() {
        warn!("Platform init warning: {}", e);
    }

    let result = match cli.command {
        Command::Run(args) => cmd_run(args),
        Command::Check(args) => cmd_check(args),
        Command::Info => cmd_info(),
    };

    if let Err(e) = result {
        error!("{:#}", e);
        eprintln!("\nerror: {:#}", e);
        process::exit(1);
    }
}

fn cmd_run(args: RunArgs) -> Result<()> {
    if !args.script.exists() {
        anyhow::bail!("Script not found: {}", args.script.display());
    }

    let source = std::fs::read_to_string(&args.script)
        .with_context(|| format!("Cannot read: {}", args.script.display()))?;

    validate_script(&args.script.to_string_lossy(), &source)?;
    debug!("Script syntax OK");

    if !args.no_benchmark {
        let clock = Clock::new();
        run_timing_benchmark(&clock)?;
    }

    info!("psychlib v{}", env!("CARGO_PKG_VERSION"));
    info!("participant : {}", args.participant);
    info!("script      : {}", args.script.display());
    info!("output      : {}", args.output.display());
    info!("seed        : {}", args.seed.map_or("entropy".into(), |s| s.to_string()));
    info!("display     : {}x{}{}", args.width, args.height, if args.fullscreen { " (fullscreen)" } else { "" });

    let experiment = ExperimentConfig {
        participant: args.participant,
        script_path: args.script,
        output_dir: args.output,
        seed: args.seed,
        render: RenderConfig {
            title: "psychlib".to_string(),
            width: args.width,
            height: args.height,
            fullscreen: args.fullscreen,
            background: psychlib::renderer::Color::BLACK,
        },
    };

    if args.headless {
        psychlib::runtime::headless_run(experiment)
    } else {
        psychlib::runtime::run(experiment)
    }
}

fn cmd_check(args: CheckArgs) -> Result<()> {
    if !args.script.exists() {
        anyhow::bail!("Script not found: {}", args.script.display());
    }
    let source = std::fs::read_to_string(&args.script)
        .with_context(|| format!("Cannot read: {}", args.script.display()))?;

    validate_script(&args.script.to_string_lossy(), &source)?;
    println!("✓  {} — OK", args.script.display());
    Ok(())
}

fn cmd_info() -> Result<()> {
    let clock = Clock::new();
    let info = clock.info();

    println!("psychlib v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Runtime");
    println!("platform       : {}", info.platform);
    println!("hi-res sleep   : {}", info.high_precision_sleep);
    println!("Luau           : mlua 0.10");
    println!("wgpu           : 22");
    println!("winit          : 0.30");
    println!();
    println!("Build");
    println!("profile        : {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    println!();

    println!("Timing self-test (20 × 10 ms sleeps)…");
    let errors = timing_selftest(&clock);
    let mean_us = errors.iter().sum::<i64>() / errors.len() as i64;
    let max_us = errors.iter().copied().max().unwrap_or(0);
    println!("mean jitter    : {} µs", mean_us);
    println!("max jitter     : {} µs", max_us);
    if max_us > 2_000 {
        println!("Max jitter > 2 ms");
    } else {
        println!("Timing OK");
    }
    Ok(())
}

fn validate_script(name: &str, source: &str) -> Result<()> {
    let lua = mlua::Lua::new();
    lua.load(source)
        .set_name(name)
        .into_function()
        .map_err(|e| anyhow::anyhow!("Luau syntax error in '{}': {}", name, e))?;
    Ok(())
}

fn run_timing_benchmark(clock: &Clock) -> Result<()> {
    info!("Timing benchmark (20 x 10 ms sleeps)…");
    let errors = timing_selftest(clock);
    let mean_us = errors.iter().sum::<i64>() / errors.len() as i64;
    let max_us = errors.iter().copied().max().unwrap_or(0);
    info!("mean jitter: {} µs  max: {} µs", mean_us, max_us);

    if max_us > 5_000 {
        anyhow::bail!(
            "Timing benchmark failed: max jitter {} µs (limit 5000 µs).\n\
             Close other applications or pass --no-benchmark to skip.",
            max_us
        );
    }
    if max_us > 2_000 {
        warn!(
            "Timing jitter elevated ({} µs)",
            max_us
        );
    } else {
        info!("Timing benchmark passed");
    }
    Ok(())
}

fn timing_selftest(clock: &Clock) -> Vec<i64> {
    let target = Duration::from_millis(10);
    (0..20)
        .map(|_| {
            let t0 = clock.now();
            clock.sleep(target);
            (clock.elapsed(t0) - target).abs().as_micros()
        })
        .collect()
}

mod platform {
    use anyhow::Result;

    pub fn init() -> Result<()> {
        #[cfg(target_os = "windows")]
        windows_timer_resolution()?;

        #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
        psychlib::set_panic_hook();

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn windows_timer_resolution() -> Result<()> {
        use std::sync::OnceLock;
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            unsafe {
                windows_sys::Win32::Media::timeBeginPeriod(1);
            }
            extern "C" fn cleanup() {
                unsafe {
                    windows_sys::Win32::Media::timeEndPeriod(1);
                }
            }
            unsafe {
                libc::atexit(cleanup);
            }
        });
        tracing::debug!("Windows timer resolution: 1 ms");
        Ok(())
    }
}
