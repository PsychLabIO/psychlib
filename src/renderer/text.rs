use crate::renderer::stimulus::{Color as StimColor, TextOptions};
use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphColor, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonRenderer, Viewport,
    cosmic_text::Align,
};
use wgpu::{Device, MultisampleState, Queue, TextureFormat};

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: GlyphonRenderer,
    viewport: Viewport,
}

impl TextRenderer {
    pub fn new(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let mut viewport = Viewport::new(device, &cache);

        viewport.update(queue, Resolution { width, height });

        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer = GlyphonRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let _ = Buffer::new(&mut font_system, Metrics::new(48.0, 57.6));

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            viewport,
        }
    }

    pub fn resize(&mut self, queue: &Queue, width: u32, height: u32) {
        self.viewport.update(queue, Resolution { width, height });
    }

    /// Prepare and immediately render a single text item into `pass`.
    pub fn draw<'pass>(
        &mut self,
        device: &Device,
        queue: &Queue,
        pass: &mut wgpu::RenderPass<'pass>,
        content: &str,
        opts: &TextOptions,
        pos: Option<(f32, f32)>,
    ) {
        let resolution = self.viewport.resolution();
        let (res_w, res_h) = (resolution.width as f32, resolution.height as f32);

        let size_px = if opts.size > 0.0 {
            opts.size
        } else {
            tracing::warn!("TextRenderer::draw called with size <= 0 using 32px fallback");
            32.0
        };
        let line_height = size_px * 1.2;

        let (sx, sy) = pos.unwrap_or((0.5, 0.5));
        let anchor_x = sx * res_w;
        let anchor_y = sy * res_h;

        let g_align = match opts.align.as_str() {
            "left" => Some(Align::Left),
            "right" => Some(Align::Right),
            _ => Some(Align::Center),
        };

        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(size_px, line_height));

        buffer.set_size(&mut self.font_system, Some(res_w), Some(res_h));

        buffer.set_text(
            &mut self.font_system,
            content,
            &Attrs::new(),
            Shaping::Advanced,
            None,
        );

        for line in buffer.lines.iter_mut() {
            line.set_align(g_align);
        }

        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut measured_width: f32 = 0.0;
        let mut line_count: usize = 0;
        for run in buffer.layout_runs() {
            measured_width = measured_width.max(run.line_w);
            line_count += 1;
        }
        let measured_height = line_count as f32 * line_height;

        let left = match opts.align.as_str() {
            "right" => anchor_x - res_w,
            "left" => anchor_x,
            _ => 0.0,
        };

        let top = anchor_y - measured_height * 0.5;

        let text_area = TextArea {
            buffer: &buffer,
            left,
            top,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: resolution.width as i32,
                bottom: resolution.height as i32,
            },
            default_color: to_glyph_color(opts.color),
            custom_glyphs: &[],
        };

        if let Err(e) = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [text_area],
            &mut self.swash_cache,
        ) {
            tracing::error!("glyphon prepare error: {e:?}");
            return;
        }

        if let Err(e) = self.renderer.render(&self.atlas, &self.viewport, pass) {
            tracing::error!("glyphon render error: {e:?}");
        }
    }
}

fn to_glyph_color(c: StimColor) -> GlyphColor {
    GlyphColor::rgba(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}
