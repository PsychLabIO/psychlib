pub mod context;
pub mod pipeline;
pub mod render_loop;
pub mod stimulus;
pub mod text;

pub use render_loop::{RenderConfig, RenderLoop};
pub use stimulus::{Color, Rect, Stimulus, TextOptions};

use crate::clock::FrameTimestamp;
use std::sync::mpsc;
use winit::event_loop::EventLoopProxy;

/// Sent through the winit proxy to wake the event loop when a render command is queued.
#[derive(Debug)]
pub struct WakeUp;

#[derive(Debug)]
pub enum RenderCommand {
    Show(Stimulus),
    Clear,
    ClearColor(Color),
    PreloadImage(String),
    Quit,
}

/// Events sent from the render thread to the script thread.
#[derive(Debug)]
pub enum RenderEvent {
    FrameFlipped(FrameTimestamp),
    ImageLoaded(String),
    ImageLoadFailed(String),
    WindowClosed,
    Error(String),
}

/// Script thread's handle to the render loop.
pub struct RenderHandle {
    pub(crate) cmd_tx: mpsc::SyncSender<RenderCommand>,
    pub(crate) event_rx: mpsc::Receiver<RenderEvent>,
    pub(crate) proxy: EventLoopProxy<WakeUp>,
    pub screen_w: f32,
    pub screen_h: f32,
}

impl RenderHandle {
    pub fn send(&self, cmd: RenderCommand) -> Result<(), String> {
        self.cmd_tx
            .send(cmd)
            .map_err(|e| format!("render channel closed: {e}"))?;
        let _ = self.proxy.send_event(WakeUp);
        Ok(())
    }

    pub fn recv(&self) -> Result<RenderEvent, String> {
        self.event_rx
            .recv()
            .map_err(|_| "render event channel closed".to_string())
    }

    pub fn try_recv(&self) -> Option<RenderEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Preload an image and block until the renderer confirms success or failure.
    pub fn preload_image(&self, path: impl Into<String>) -> Result<String, String> {
        let path = path.into();
        self.send(RenderCommand::PreloadImage(path))?;
        loop {
            match self.recv()? {
                RenderEvent::ImageLoaded(p) => return Ok(p),
                RenderEvent::ImageLoadFailed(p) => {
                    return Err(format!("failed to load image: {p}"));
                }
                RenderEvent::WindowClosed => return Err("window closed".into()),
                RenderEvent::Error(e) => return Err(e),
                RenderEvent::FrameFlipped(_) => continue,
            }
        }
    }

    /// Send `Show(stim)`, block until `FrameFlipped`.
    pub fn show_and_wait_flip(&self, stim: Stimulus) -> Result<crate::clock::Instant, String> {
        self.send(RenderCommand::Show(stim))?;
        loop {
            match self.recv()? {
                RenderEvent::FrameFlipped(ts) => return Ok(ts.instant),
                RenderEvent::WindowClosed => return Err("window closed".into()),
                RenderEvent::Error(e) => return Err(e),
                RenderEvent::ImageLoaded(_) | RenderEvent::ImageLoadFailed(_) => continue,
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
                RenderEvent::ImageLoaded(_) | RenderEvent::ImageLoadFailed(_) => continue,
            }
        }
    }

    pub fn screen_size(&self) -> Option<(f32, f32)> {
        Some((self.screen_w, self.screen_h))
    }
}
