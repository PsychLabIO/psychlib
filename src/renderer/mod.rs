pub mod context;
pub mod pipeline;
pub mod render_loop;
pub mod stimulus;

pub use render_loop::{RenderConfig, RenderLoop};
pub use stimulus::{Color, Rect, Stimulus, TextOptions};

use crate::clock::FrameTimestamp;
use std::sync::mpsc;

/// Commands sent from the script thread to the render thread.
#[derive(Debug)]
pub enum RenderCommand {
    Show(Stimulus),
    Clear,
    ClearColor(Color),
    Quit,
}

/// Events sent from the render thread to the script thread.
#[derive(Debug)]
pub enum RenderEvent {
    FrameFlipped(FrameTimestamp),
    WindowClosed,
    Error(String),
}

/// Script thread's handle to the render loop.
pub struct RenderHandle {
    pub cmd_tx: mpsc::SyncSender<RenderCommand>,
    pub event_rx: mpsc::Receiver<RenderEvent>,
}

impl RenderHandle {
    pub fn send(&self, cmd: RenderCommand) -> Result<(), String> {
        self.cmd_tx
            .send(cmd)
            .map_err(|e| format!("render channel closed: {e}"))
    }

    pub fn recv(&self) -> Result<RenderEvent, String> {
        self.event_rx
            .recv()
            .map_err(|_| "render event channel closed".to_string())
    }

    pub fn try_recv(&self) -> Option<RenderEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Send `Show(stim)`, block until `FrameFlipped`
    pub fn show_and_wait_flip(&self, stim: Stimulus) -> Result<crate::clock::Instant, String> {
        self.send(RenderCommand::Show(stim))?;
        loop {
            match self.recv()? {
                RenderEvent::FrameFlipped(ts) => return Ok(ts.instant),
                RenderEvent::WindowClosed => return Err("window closed".into()),
                RenderEvent::Error(e) => return Err(e),
            }
        }
    }

    /// Send `Clear`, block until `FrameFlipped`.
    pub fn clear_and_wait_flip(&self) -> Result<crate::clock::Instant, String> {
        self.send(RenderCommand::Clear)?;
        loop {
            match self.recv()? {
                RenderEvent::FrameFlipped(ts) => return Ok(ts.instant),
                RenderEvent::WindowClosed => return Err("window closed".into()),
                RenderEvent::Error(e) => return Err(e),
            }
        }
    }
}
