use crate::clock::Clock;
use crate::renderer::{
    context::WgpuContext,
    pipeline::ColorPipeline,
    stimulus::{Color, Stimulus},
    RenderCommand, RenderEvent, RenderHandle,
};
use std::sync::{mpsc, Arc};
use tracing::{debug, error, info, warn};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowId},
};

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub background: Color,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            title: "psychlib".to_string(),
            width: 1024,
            height: 768,
            fullscreen: false,
            background: Color::BLACK,
        }
    }
}

pub struct RenderLoop {
    config: RenderConfig,
    clock: Clock,
    cmd_rx: mpsc::Receiver<RenderCommand>,
    event_tx: mpsc::SyncSender<RenderEvent>,
    window: Option<Arc<Window>>,
    ctx: Option<WgpuContext>,
    pipeline: Option<ColorPipeline>,
    current: Option<Stimulus>,
}

impl RenderLoop {
    pub fn create(config: RenderConfig, clock: Clock) -> (RenderHandle, EventLoop<()>, Self) {
        let (cmd_tx, cmd_rx) = mpsc::sync_channel(8);
        let (event_tx, event_rx) = mpsc::sync_channel(8);

        let handle = RenderHandle { cmd_tx, event_rx };

        let event_loop = EventLoop::new().expect("Failed to create EventLoop");
        event_loop.set_control_flow(ControlFlow::Poll);

        let rl = Self {
            config,
            clock,
            cmd_rx,
            event_tx,
            window: None,
            ctx: None,
            pipeline: None,
            current: None,
        };
        (handle, event_loop, rl)
    }

    fn render(&mut self, stim: &Stimulus) {
        let (ctx, pipeline) = match (self.ctx.as_ref(), self.pipeline.as_ref()) {
            (Some(c), Some(p)) => (c, p),
            _ => return,
        };

        if !ctx.configured {
            return;
        }

        let output = match ctx.surface.get_current_texture() {
            Ok(o) => o,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                if let Some(c) = self.ctx.as_mut() {
                    c.resize(c.size.width, c.size.height);
                }
                return;
            }
            Err(e) => {
                error!("Surface error: {e}");
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        let bg = self.config.background;
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: bg.r as f64,
                            g: bg.g as f64,
                            b: bg.b as f64,
                            a: bg.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.draw(&mut pass, &ctx.queue, &ctx.device, pipeline, stim);
        }

        ctx.queue.submit(std::iter::once(enc.finish()));
        output.present();

        let ts = self.clock.record_frame("flip");
        debug!("frame flip: {}", ts.instant);
        let _ = self.event_tx.try_send(RenderEvent::FrameFlipped(ts));
    }

    fn draw(
        &self,
        pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        pipeline: &ColorPipeline,
        stim: &Stimulus,
    ) {
        match stim {
            Stimulus::Rect { rect, color } => {
                pipeline.draw_quad(
                    pass,
                    queue,
                    device,
                    rect.cx,
                    rect.cy,
                    rect.hw,
                    rect.hh,
                    [color.r, color.g, color.b, color.a],
                );
            }
            Stimulus::Fixation {
                color,
                arm_len,
                thickness,
            } => {
                pipeline.draw_quad(
                    pass,
                    queue,
                    device,
                    0.0,
                    0.0,
                    *arm_len,
                    *thickness,
                    [color.r, color.g, color.b, color.a],
                );
                pipeline.draw_quad(
                    pass,
                    queue,
                    device,
                    0.0,
                    0.0,
                    *thickness,
                    *arm_len,
                    [color.r, color.g, color.b, color.a],
                );
            }
            Stimulus::Text { opts, .. } => {
                warn!("Text stimulus: glyphon not yet integrated");
                let c = &opts.color;
                pipeline.draw_quad(
                    pass,
                    queue,
                    device,
                    0.0,
                    0.0,
                    0.1,
                    0.05,
                    [c.r, c.g, c.b, c.a],
                );
            }
            Stimulus::Image { rect, tint, .. } => {
                warn!("Image stimulus: texture pipeline not yet integrated");
                pipeline.draw_quad(
                    pass,
                    queue,
                    device,
                    rect.cx,
                    rect.cy,
                    rect.hw,
                    rect.hh,
                    [tint.r, 0.0, tint.b, tint.a],
                );
            }
            Stimulus::Composite(parts) => {
                for s in parts {
                    self.draw(pass, queue, device, pipeline, s);
                }
            }
        }
    }
}

impl ApplicationHandler for RenderLoop {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attrs = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width,
                self.config.height,
            ));

        if self.config.fullscreen {
            attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );
        window.set_cursor_visible(false);

        let ctx = pollster::block_on(WgpuContext::new(window.clone()))
            .expect("Failed to initialise wgpu");

        let pipeline = ColorPipeline::new(&ctx.device, ctx.config.format);

        info!(
            "Render window ready: {}x{}",
            ctx.size.width, ctx.size.height
        );

        self.window = Some(window);
        self.ctx = Some(ctx);
        self.pipeline = Some(pipeline);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                let _ = self.event_tx.try_send(RenderEvent::WindowClosed);
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(ctx) = self.ctx.as_mut() {
                    ctx.resize(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                loop {
                    match self.cmd_rx.try_recv() {
                        Ok(RenderCommand::Show(stim)) => {
                            self.current = Some(stim.clone());
                            self.render(&stim);
                        }
                        Ok(RenderCommand::Clear) => {
                            self.current = None;
                            self.render(&Stimulus::blank(self.config.background));
                        }
                        Ok(RenderCommand::ClearColor(c)) => {
                            self.current = None;
                            self.render(&Stimulus::blank(c));
                        }
                        Ok(RenderCommand::Quit) => {
                            event_loop.exit();
                            return;
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            event_loop.exit();
                            return;
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                    }
                }

                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}
