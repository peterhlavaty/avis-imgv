//! Building the smaller copies the GPU needs to shrink a texture cleanly.
//!
//! A photograph is nearly always drawn smaller than it is stored: a 6000 pixel
//! image zoomed to half is three source pixels per screen pixel, and a bilinear
//! sampler only ever reads four of every nine. What it misses is exactly the
//! fine detail — grass, fabric, hair — so it sparkles and crawls as the image
//! is panned.
//!
//! The fix is the standard one: store the texture pre-shrunk at every halving
//! and let the sampler blend between the two levels either side of the size
//! being drawn. The levels are made on the GPU, one render pass each, which
//! costs a fraction of a millisecond for the whole chain.

use eframe::wgpu;

/// The format every image is uploaded in.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// How many levels a texture of this size has, counting the image itself.
///
/// Down to a single pixel: the chain is a third of the image again however far
/// it goes, and stopping early only leaves the smallest sizes aliasing.
pub fn levels(width: u32, height: u32) -> u32 {
    32 - width.max(height).max(1).leading_zeros()
}

/// Halves a dimension, never reaching zero, the way the GPU does between mip
/// levels. Used by the tests to check the chain reaches the bottom.
#[cfg_attr(not(test), allow(dead_code))]
fn halved(size: u32) -> u32 {
    (size / 2).max(1)
}

/// Builds the mip chain of a texture by repeatedly halving it on the GPU.
pub struct MipGenerator {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl MipGenerator {
    pub fn new(device: &wgpu::Device) -> MipGenerator {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("avis mipmap"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("avis mipmap"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("avis mipmap"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("avis mipmap"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                targets: &[Some(FORMAT.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("avis mipmap"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        MipGenerator {
            pipeline,
            layout,
            sampler,
        }
    }

    /// Fills every level of `texture` below the first from the one above it.
    ///
    /// Level zero is expected to already hold the image.
    pub fn generate(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        levels: u32,
    ) {
        if levels < 2 {
            return;
        }

        let views: Vec<wgpu::TextureView> = (0..levels)
            .map(|level| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("avis mipmap level"),
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("avis mipmap"),
        });

        for level in 1..levels as usize {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("avis mipmap"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&views[level - 1]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("avis mipmap"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &views[level],
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

/// A full screen triangle that samples the level above.
///
/// Three vertices rather than a quad's six, with the texture coordinates
/// derived from the vertex index, so no buffers are needed at all.
const SHADER: &str = r#"
struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> Varyings {
    var out: Varyings;

    out.uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    out.position = vec4<f32>(out.uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);

    return out;
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@fragment
fn fragment(in: Varyings) -> @location(0) vec4<f32> {
    return textureSample(source, source_sampler, in.uv);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_pixel_has_one_level() {
        assert_eq!(levels(1, 1), 1);
    }

    #[test]
    fn every_halving_is_a_level() {
        assert_eq!(levels(2, 1), 2);
        assert_eq!(levels(4, 4), 3);
        assert_eq!(levels(1024, 1024), 11);
    }

    #[test]
    fn the_longer_edge_decides() {
        assert_eq!(levels(1024, 1), levels(1024, 1024));
    }

    #[test]
    fn a_size_that_is_not_a_power_of_two_rounds_down() {
        // 6000 halves to 2 pixels eleven times and to one on the twelfth.
        assert_eq!(levels(6000, 4000), 13);
    }

    #[test]
    fn halving_never_reaches_zero() {
        assert_eq!(halved(1), 1);
        assert_eq!(halved(3), 1);
        assert_eq!(halved(4), 2);
    }

    #[test]
    fn the_chain_reaches_a_single_pixel() {
        let (mut width, mut height) = (6000u32, 4000u32);
        for _ in 1..levels(width, height) {
            width = halved(width);
            height = halved(height);
        }

        assert_eq!((width, height), (1, 1));
    }
}
