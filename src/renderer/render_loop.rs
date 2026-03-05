use crate::clock::Clock;
use crate::renderer::{
    RenderCommand, RenderEvent, RenderHandle, WakeUp,
    context::WgpuContext,
    pipeline::{ColorPipeline, DrawImageOutcome, TexturePipeline},
    stimulus::{Color, Stimulus},
    text::TextRenderer,
};
use std::sync::{Arc, mpsc};
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
    pipeline: Option<Arc<ColorPipeline>>,
    texture_pipeline: Option<TexturePipeline>,
    text_renderer: Option<TextRenderer>,
    current: Option<Stimulus>,
    dirty: bool,
}

impl RenderLoop {
    pub fn create(config: RenderConfig, clock: Clock) -> (RenderHandle, EventLoop<WakeUp>, Self) {
        let (cmd_tx, cmd_rx) = mpsc::sync_channel(8);
        let (event_tx, event_rx) = mpsc::sync_channel(8);

        let event_loop = EventLoop::<WakeUp>::with_user_event()
            .build()
            .expect("Failed to create EventLoop");
        event_loop.set_control_flow(ControlFlow::Wait);

        let proxy = event_loop.create_proxy();

        let handle = RenderHandle {
            cmd_tx,
            event_rx,
            proxy,
        };

        let rl = Self {
            config,
            clock,
            cmd_rx,
            event_tx,
            window: None,
            ctx: None,
            pipeline: None,
            texture_pipeline: None,
            text_renderer: None,
            current: None,
            dirty: false,
        };
        (handle, event_loop, rl)
    }

    fn preload_image(&mut self, path: String) {
        let Some(ctx) = self.ctx.as_ref() else {
            error!("preload_image called before GPU context is ready");
            let _ = self.event_tx.try_send(RenderEvent::ImageLoadFailed(path));
            return;
        };

        let Some(tp) = self.texture_pipeline.as_mut() else {
            error!("preload_image called before texture pipeline is ready");
            let _ = self.event_tx.try_send(RenderEvent::ImageLoadFailed(path));
            return;
        };

        let ok = tp.preload(&ctx.device, &ctx.queue, &path);
        let evt = if ok {
            RenderEvent::ImageLoaded(path)
        } else {
            RenderEvent::ImageLoadFailed(path)
        };
        let _ = self.event_tx.try_send(evt);
    }

    fn render(&mut self, stim: &Stimulus) {
        let Some(ctx) = self.ctx.as_ref() else { return };

        if ctx.size.width == 0 || ctx.size.height == 0 {
            return;
        }

        let device = ctx.device.clone();
        let queue = ctx.queue.clone();

        let surface_texture = match ctx.surface.get_current_texture() {
            Ok(o) => o,
            Err(wgpu::SurfaceError::Lost) => {
                let (w, h) = (ctx.size.width, ctx.size.height);
                self.ctx.as_mut().unwrap().resize(w, h);
                return;
            }
            Err(wgpu::SurfaceError::Outdated) => return,
            Err(e) => {
                error!("Surface error: {e}");
                return;
            }
        };

        let pipeline = match self.pipeline.clone() {
            Some(p) => p,
            None => return,
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame"),
        });

        let bg = self.config.background;
        let stim = stim.clone();

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
                multiview_mask: None,
            });

            self.draw(&mut pass, &device, &queue, &pipeline, &stim);
        }

        queue.submit(std::iter::once(enc.finish()));
        surface_texture.present();

        let ts = self.clock.record_frame("flip");
        debug!("frame flip: {}", ts.instant);
        let _ = self.event_tx.try_send(RenderEvent::FrameFlipped(ts));
    }

    fn draw<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pipeline: &'pass ColorPipeline,
        stim: &Stimulus,
    ) {
        match stim {
            Stimulus::Rect { rect, color } => {
                pipeline.draw_quad(
                    pass,
                    device,
                    rect.cx,
                    rect.cy,
                    rect.hw,
                    rect.hh,
                    color.to_array(),
                );
            }

            Stimulus::Fixation {
                color,
                arm_len,
                thickness,
            } => {
                let aspect = self
                    .ctx
                    .as_ref()
                    .map(|ctx| ctx.size.width as f32 / ctx.size.height as f32)
                    .unwrap_or(1.0);
                
                pipeline.draw_quad(
                    pass,
                    device,
                    0.0,
                    0.0,
                    *arm_len,
                    *thickness * aspect,
                    color.to_array(),
                );

                pipeline.draw_quad(
                    pass,
                    device,
                    0.0,
                    0.0,
                    *thickness * aspect,
                    *arm_len,
                    color.to_array(),
                );
            }

            Stimulus::Text { content, opts, pos } => {
                if let Some(tr) = self.text_renderer.as_mut() {
                    tr.draw(device, queue, pass, content, opts, *pos);
                }
            }

            Stimulus::Image { path, rect, tint } => {
                let outcome = if let Some(tp) = self.texture_pipeline.as_mut() {
                    tp.draw_image(
                        pass,
                        queue,
                        path,
                        rect.cx,
                        rect.cy,
                        rect.hw,
                        rect.hh,
                        tint.to_array(),
                    )
                } else {
                    warn!("Image stimulus: texture pipeline not initialised");
                    DrawImageOutcome::Fallback {
                        cx: rect.cx,
                        cy: rect.cy,
                        hw: rect.hw,
                        hh: rect.hh,
                    }
                };

                if let DrawImageOutcome::Fallback { cx, cy, hw, hh } = outcome {
                    pipeline.draw_quad(pass, device, cx, cy, hw, hh, Color::MAGENTA.to_array());
                }
            }

            Stimulus::Composite(parts) => {
                let parts = parts.clone();
                for s in &parts {
                    self.draw(pass, device, queue, pipeline, s);
                }
            }
        }
    }
}

impl ApplicationHandler<WakeUp> for RenderLoop {
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

        let pipeline = Arc::new(ColorPipeline::new(&ctx.device, ctx.config.format));
        let texture_pipeline = TexturePipeline::new(&ctx.device, ctx.config.format);

        let text_renderer = TextRenderer::new(
            &ctx.device,
            &ctx.queue,
            ctx.config.format,
            ctx.size.width,
            ctx.size.height,
        );

        info!(
            "Render window ready: {}x{}",
            ctx.size.width, ctx.size.height
        );

        self.window = Some(window);
        self.ctx = Some(ctx);
        self.pipeline = Some(pipeline);
        self.texture_pipeline = Some(texture_pipeline);
        self.text_renderer = Some(text_renderer);
    }

    fn user_event(&mut self, _: &ActiveEventLoop, _: WakeUp) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                use crate::io::keyboard::{KeyState, push_key_event};
                use winit::event::ElementState;

                if let Some(code) = crate::io::keyboard::KeyCode::from_winit(&event.logical_key) {
                    let state = match event.state {
                        ElementState::Pressed => KeyState::Pressed,
                        ElementState::Released => KeyState::Released,
                    };
                    push_key_event(code, state);
                }
            }

            WindowEvent::CloseRequested => {
                info!("Window close requested");
                let _ = self.event_tx.try_send(RenderEvent::WindowClosed);
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(ctx) = self.ctx.as_mut() {
                        ctx.resize(size.width, size.height);
                    }
                    if let (Some(ctx), Some(tr)) = (self.ctx.as_ref(), self.text_renderer.as_mut())
                    {
                        tr.resize(&ctx.queue, size.width, size.height);
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                let mut needs_redraw = false;

                loop {
                    match self.cmd_rx.try_recv() {
                        Ok(RenderCommand::Show(stim)) => {
                            self.current = Some(stim);
                            self.dirty = true;
                            needs_redraw = true;
                        }
                        Ok(RenderCommand::Clear) => {
                            self.current = None;
                            self.dirty = true;
                            needs_redraw = true;
                        }
                        Ok(RenderCommand::ClearColor(c)) => {
                            self.current = None;
                            self.config.background = c;
                            self.dirty = true;
                            needs_redraw = true;
                        }
                        Ok(RenderCommand::PreloadImage(path)) => {
                            self.preload_image(path);
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

                if needs_redraw || self.dirty {
                    let stim = self
                        .current
                        .clone()
                        .unwrap_or_else(|| Stimulus::blank(self.config.background));

                    self.render(&stim);
                    self.dirty = false;
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) { /* no op */
    }
}
