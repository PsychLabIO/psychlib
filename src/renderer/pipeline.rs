use image::GenericImageView;
use std::collections::{HashMap, HashSet};
use tracing::{error, info, warn};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub pos: [f32; 2],
}

impl ColorVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const UNIT_VERTS: &[ColorVertex] = &[
    ColorVertex { pos: [-1.0, 1.0] },
    ColorVertex { pos: [1.0, 1.0] },
    ColorVertex { pos: [1.0, -1.0] },
    ColorVertex { pos: [-1.0, -1.0] },
];

const UNIT_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ColorUniforms {
    transform: [f32; 4],
    color: [f32; 4],
}

const COLOR_SHADER: &str = r#"
struct Uniforms {
    transform: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs(@location(0) pos: vec2<f32>) -> VsOut {
    var out: VsOut;

    let cx = u.transform.x;
    let cy = u.transform.y;
    let hw = u.transform.z;
    let hh = u.transform.w;

    let scaled     = vec2<f32>(pos.x * hw, pos.y * hh);
    let translated = scaled + vec2<f32>(cx, cy);

    out.pos = vec4<f32>(translated, 0.0, 1.0);
    return out;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    return u.color;
}
"#;

pub struct ColorPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
}

impl ColorPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("color_shader"),
            source: wgpu::ShaderSource::Wgsl(COLOR_SHADER.into()),
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            size: std::mem::size_of::<ColorUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("color_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("color_bind_group"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("color_pipeline_layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("color_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[ColorVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("unit_quad_vertex_buffer"),
            contents: bytemuck::cast_slice(UNIT_VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("unit_quad_index_buffer"),
            contents: bytemuck::cast_slice(UNIT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            uniform_buf,
            bind_group,
            vertex_buf,
            index_buf,
        }
    }

    pub fn draw_quad(
        &self,
        pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        cx: f32,
        cy: f32,
        hw: f32,
        hh: f32,
        color: [f32; 4],
    ) {
        let uniforms = ColorUniforms {
            transform: [cx, cy, hw, hh],
            color,
        };

        queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..1);
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureVertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl TextureVertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<TextureVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
    };
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TintUniform {
    color: [f32; 4],
}

struct GpuTexture {
    bind_group: wgpu::BindGroup,
}

pub enum DrawImageOutcome {
    Ok,
    Fallback { cx: f32, cy: f32, hw: f32, hh: f32 },
}

const TEXTURE_SHADER: &str = r#"
struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv:  vec2<f32>,
};
struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0)       uv:   vec2<f32>,
};

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip = vec4<f32>(v.pos, 0.0, 1.0);
    out.uv   = v.uv;
    return out;
}

@group(0) @binding(0) var t_image: texture_2d<f32>;
@group(0) @binding(1) var s_image: sampler;
@group(1) @binding(0) var<uniform> tint: vec4<f32>;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(t_image, s_image, in.uv) * tint;
}
"#;

pub struct TexturePipeline {
    pipeline: wgpu::RenderPipeline,
    texture_bgl: wgpu::BindGroupLayout,
    tint_bgl: wgpu::BindGroupLayout,
    vertex_buf: wgpu::Buffer,
    tint_buf: wgpu::Buffer,
    tint_bg: wgpu::BindGroup,
    cache: HashMap<String, GpuTexture>,
    failed: HashSet<String>,
}

impl TexturePipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("texture_shader"),
            source: wgpu::ShaderSource::Wgsl(TEXTURE_SHADER.into()),
        });

        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let tint_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tint_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("texture_pipeline_layout"),
            bind_group_layouts: &[&texture_bgl, &tint_bgl],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextureVertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_vbuf"),
            size: (6 * std::mem::size_of::<TextureVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let tint_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tint_ubuf"),
            contents: bytemuck::bytes_of(&TintUniform {
                color: [1.0, 1.0, 1.0, 1.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let tint_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tint_bg"),
            layout: &tint_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: tint_buf.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            texture_bgl,
            tint_bgl,
            vertex_buf,
            tint_buf,
            tint_bg,
            cache: HashMap::new(),
            failed: HashSet::new(),
        }
    }

    pub fn preload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> bool {
        if self.cache.contains_key(path) {
            return true;
        }
        if self.failed.contains(path) {
            return false;
        }

        match self.load_texture(device, queue, path) {
            Some(gt) => {
                info!("TexturePipeline: preloaded '{path}'");
                self.cache.insert(path.to_string(), gt);
                true
            }
            None => {
                error!(
                    "TexturePipeline: preload failed for '{path}'; \
                     magenta fallback will be shown at draw time"
                );
                self.failed.insert(path.to_string());
                false
            }
        }
    }

    /// Draw the preloaded image at `path` scaled to the quad `(cx, cy, hw, hh)`.
    pub fn draw_image<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        queue: &wgpu::Queue,
        path: &str,
        cx: f32,
        cy: f32,
        hw: f32,
        hh: f32,
        tint: [f32; 4],
    ) -> DrawImageOutcome {
        if !self.cache.contains_key(path) {
            if !self.failed.contains(path) {
                warn!(
                    "TexturePipeline: draw_image called for '{path}' \
                     which was never preloaded — showing magenta fallback"
                );
            }
            return DrawImageOutcome::Fallback { cx, cy, hw, hh };
        }

        let gpu_tex = self.cache.get(path).unwrap();

        let (l, r) = (cx - hw, cx + hw);
        let (b, t) = (cy - hh, cy + hh);

        let verts: [TextureVertex; 6] = [
            TextureVertex {
                pos: [l, t],
                uv: [0.0, 0.0],
            },
            TextureVertex {
                pos: [r, t],
                uv: [1.0, 0.0],
            },
            TextureVertex {
                pos: [l, b],
                uv: [0.0, 1.0],
            },
            TextureVertex {
                pos: [l, b],
                uv: [0.0, 1.0],
            },
            TextureVertex {
                pos: [r, t],
                uv: [1.0, 0.0],
            },
            TextureVertex {
                pos: [r, b],
                uv: [1.0, 1.0],
            },
        ];
        queue.write_buffer(&self.vertex_buf, 0, bytemuck::cast_slice(&verts));
        queue.write_buffer(
            &self.tint_buf,
            0,
            bytemuck::bytes_of(&TintUniform { color: tint }),
        );

        let pipeline = &self.pipeline;
        let tex_bg = &gpu_tex.bind_group;
        let tint_bg = &self.tint_bg;
        let vertex_buf = self.vertex_buf.slice(..);

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, tex_bg, &[]);
        pass.set_bind_group(1, tint_bg, &[]);
        pass.set_vertex_buffer(0, vertex_buf);
        pass.draw(0..6, 0..1);

        DrawImageOutcome::Ok
    }

    fn load_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
    ) -> Option<GpuTexture> {
        let img = match image::open(path) {
            Ok(i) => i,
            Err(e) => {
                warn!("image::open('{path}'): {e}");
                return None;
            }
        };

        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some(path),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &rgba,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("image_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bg"),
            layout: &self.texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Some(GpuTexture { bind_group })
    }
}
