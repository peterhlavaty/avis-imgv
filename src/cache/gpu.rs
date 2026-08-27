//! GPU residency: uploading decoded pixels into textures and keeping a bounded
//! number of them alive.
//!
//! An upload is a straight copy of the RGBA8 buffer the decoder produced, so
//! the images the user is about to reach are already sitting in VRAM when the
//! frame that draws them starts.

use std::collections::HashMap;

use eframe::egui_wgpu::RenderState;
use eframe::wgpu;
use epaint::{TextureId, Vec2};

use crate::decoder::{DecodedImage, BYTES_PER_PIXEL};

use super::policy;

/// A texture owned by the viewer, freed when dropped.
pub struct GpuTexture {
    pub id: TextureId,
    pub size: Vec2,
    render_state: RenderState,
}

impl Drop for GpuTexture {
    fn drop(&mut self) {
        self.render_state.renderer.write().free_texture(&self.id);
    }
}

/// Textures resident on the GPU, keyed by image index.
pub struct GpuCache {
    entries: HashMap<usize, GpuTexture>,
    render_state: RenderState,
    capacity: usize,
}

impl GpuCache {
    /// `capacity` is the number of textures to keep resident.
    pub fn new(render_state: RenderState, capacity: usize) -> GpuCache {
        GpuCache {
            entries: HashMap::new(),
            render_state,
            capacity: capacity.max(1),
        }
    }

    /// Largest texture edge the adapter supports.
    pub fn max_texture_edge(&self) -> u32 {
        self.render_state.adapter.limits().max_texture_dimension_2d
    }

    pub fn get(&self, index: usize) -> Option<&GpuTexture> {
        self.entries.get(&index)
    }

    pub fn contains(&self, index: usize) -> bool {
        self.entries.contains_key(&index)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
    }

    /// Uploads `image` and evicts the texture furthest from `cursor` if the
    /// cache is over capacity.
    pub fn upload(&mut self, index: usize, image: &DecodedImage, cursor: usize, total: usize) {
        let Some(texture) = self.create_texture(image) else {
            return;
        };

        self.entries.insert(index, texture);
        self.evict_until_within_capacity(index, cursor, total);
    }

    pub fn remove(&mut self, index: usize) {
        self.entries.remove(&index);
    }

    /// Removes a texture whose image has left the collection, shifting the
    /// entries above it down.
    pub fn remove_shifting(&mut self, index: usize) {
        policy::remove_and_shift(&mut self.entries, index);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Drops every texture whose index is not in `keep`.
    pub fn retain(&mut self, keep: impl Fn(usize) -> bool) {
        self.entries.retain(|index, _| keep(*index));
    }

    fn evict_until_within_capacity(&mut self, keep: usize, cursor: usize, total: usize) {
        while self.entries.len() > self.capacity {
            match policy::furthest(self.entries.keys().copied(), cursor, total, keep) {
                Some(victim) => self.remove(victim),
                None => return,
            }
        }
    }

    fn create_texture(&self, image: &DecodedImage) -> Option<GpuTexture> {
        if image.width == 0 || image.height == 0 {
            return None;
        }

        let size = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };

        let texture = self
            .render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(image.file_name()),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

        self.render_state.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width * BYTES_PER_PIXEL as u32),
                rows_per_image: Some(image.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let id = self.render_state.renderer.write().register_native_texture(
            &self.render_state.device,
            &view,
            wgpu::FilterMode::Linear,
        );

        Some(GpuTexture {
            id,
            size: Vec2::new(image.width as f32, image.height as f32),
            render_state: self.render_state.clone(),
        })
    }
}
