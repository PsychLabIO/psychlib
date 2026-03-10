use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Clock error: {0}")]
    Clock(String),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Script error: {0}")]
    Script(#[from] mlua::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Timing overflow - experiment ran longer than u64::MAX nanoseconds")]
    TimingOverflow,
}

pub type Result<T> = std::result::Result<T, Error>;
