use crate::renderer::stimulus::{Color as StimColor, TextOptions};
use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphColor, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonRenderer, Viewport,
};
use wgpu::{Device, MultisampleState, Queue, TextureFormat};

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: GlyphonRenderer,
    buffer: Buffer,
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

        let height_f = height as f32;
        let mut buffer = Buffer::new(
            &mut font_system,
            Metrics::new(height_f * 0.05, height_f * 0.05 * 1.2),
        );
        buffer.set_size(&mut font_system, Some(width as f32), Some(height as f32));

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            buffer,
            viewport,
        }
    }

    pub fn resize(&mut self, queue: &Queue, width: u32, height: u32) {
        self.viewport.update(queue, Resolution { width, height });
        self.buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32),
        );
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        content: &str,
        opts: &TextOptions,
        pos: Option<(f32, f32)>,
    ) {
        let resolution = self.viewport.resolution();

        let size_px = opts.size * resolution.height as f32;
        let line_height = size_px * 1.2;

        self.buffer
            .set_metrics(&mut self.font_system, Metrics::new(size_px, line_height));

        self.buffer.set_text(
            &mut self.font_system,
            content,
            &Attrs::new(),
            Shaping::Advanced,
            None,
        );

        self.buffer.shape_until_scroll(&mut self.font_system, false);

        let mut measured_width: f32 = 0.0;
        for run in self.buffer.layout_runs() {
            measured_width = measured_width.max(run.line_w);
        }

        let (x_px, y_px) = if let Some((nx, ny)) = pos {
            let x = ((nx + 1.0) * 0.5) * resolution.width as f32;
            let y = ((1.0 - ny) * 0.5) * resolution.height as f32;
            (x, y)
        } else {
            (
                resolution.width as f32 * 0.5,
                resolution.height as f32 * 0.5,
            )
        };

        let color = to_glyph_color(opts.color);

        let text_area = TextArea {
            buffer: &self.buffer,
            left: match opts.align.as_str() {
                "center" => x_px - measured_width * 0.5,
                "right" => x_px - measured_width,
                _ => x_px,
            },
            top: y_px,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: resolution.width as i32,
                bottom: resolution.height as i32,
            },
            default_color: color,
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
        }
    }

    pub fn render<'pass>(&self, pass: &mut wgpu::RenderPass<'pass>) {
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
