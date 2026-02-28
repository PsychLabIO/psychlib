#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub use console_error_panic_hook::set_once as set_panic_hook;

pub mod error;

pub mod clock;
pub mod data;
pub mod io;
pub mod renderer;
pub mod runtime;
pub mod scheduler;
pub mod script;
pub mod tests;

pub use clock::{Clock, Duration, FrameTimestamp, Instant};
pub use data::{CsvWriter, DataStore, JsonWriter, SessionHeader, TrialRecord};
pub use error::Error;
pub use io::{
    InputEvent, InputKind, KeyCode, KeyState, MouseButton, Response, ResponseOutcome,
    ResponseWindow,
};
pub use runtime::{ExperimentConfig, headless_run};
pub use scheduler::{Event, EventKind, Scheduler, TrialPhase};
pub use script::ScriptHost;
