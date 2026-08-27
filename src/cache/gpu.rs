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

use crate::decoder::preview::Preview;
use crate::decoder::{DecodedImage, Surface, BYTES_PER_PIXEL};
use crate::metadata::Orientation;

use super::policy;

/// A texture owned by the viewer, freed when dropped.
pub struct GpuTexture {
    pub id: TextureId,
    /// Size the image is shown at, which a quarter turn swaps.
    pub size: Vec2,
    /// The turn the rasteriser still has to make.
    pub orientation: Orientation,
    /// How much of the image's own resolution these texels hold: one for the
    /// image itself, less for a reduced copy or a camera thumbnail.
    ///
    /// The view reads it to notice that it is drawing the image larger than
    /// what has been uploaded, and asks for the rest.
    pub resolution: f32,
    render_state: RenderState,
}

/// One texture's worth of pixels and what they stand for.
struct Upload<'a> {
    pixels: &'a [u8],
    width: u32,
    height: u32,
    /// Size the image is shown at, whatever resolution these pixels are.
    shown: Vec2,
    resolution: f32,
    orientation: Orientation,
    label: &'a str,
}

impl GpuTexture {
    /// Whether these texels are the image itself rather than a reduced copy.
    pub fn is_full(&self) -> bool {
        self.resolution >= 1.0
    }

    /// Whether drawing the image `width` points across would magnify what has
    /// been uploaded.
    pub fn is_short_for(&self, width: f32) -> bool {
        is_short(self.resolution, self.size.x, width)
    }
}

/// Whether a texture holding `resolution` of an image shown `shown` wide is
/// being magnified by drawing it `drawn` wide.
///
/// The tolerance keeps a copy that is a rounded pixel short of the drawn size
/// from asking for the full image on every frame.
fn is_short(resolution: f32, shown: f32, drawn: f32) -> bool {
    resolution < 1.0 && drawn > shown * resolution * 1.05
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

    /// Uploads `image` at screen resolution and evicts the texture furthest
    /// from `cursor` if the cache is over capacity.
    pub fn upload(&mut self, index: usize, image: &DecodedImage, cursor: usize, total: usize) {
        let upload = describe(image, image.for_upload(), image.upload_resolution());
        self.put(index, upload, cursor, total);
    }

    /// Uploads `image` at its own resolution, for when the user has zoomed in
    /// past what the reduced copy can show.
    pub fn upload_full(&mut self, index: usize, image: &DecodedImage, cursor: usize, total: usize) {
        let upload = describe(image, &image.full, 1.0);
        self.put(index, upload, cursor, total);
    }

    /// Uploads a camera thumbnail that stands in for a larger image.
    ///
    /// It reports the size of the image it stands for rather than its own, so
    /// the layout is already right and nothing moves when the real one lands.
    pub fn upload_preview(&mut self, index: usize, preview: &Preview, cursor: usize, total: usize) {
        let Some(image) = &preview.image else {
            return;
        };

        let full = Vec2::new(preview.full_size.0 as f32, preview.full_size.1 as f32);
        let shown = crate::view::texture::displayed_size(full, preview.orientation);

        self.put(
            index,
            Upload {
                pixels: image,
                width: image.width(),
                height: image.height(),
                shown,
                resolution: image.width() as f32 / preview.full_size.0.max(1) as f32,
                orientation: preview.orientation,
                label: "preview",
            },
            cursor,
            total,
        );
    }

    /// Uploads pixels and makes room for them.
    fn put(&mut self, index: usize, upload: Upload<'_>, cursor: usize, total: usize) {
        let Some(texture) = self.create_texture(upload) else {
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

    fn create_texture(&self, upload: Upload<'_>) -> Option<GpuTexture> {
        let Upload {
            pixels,
            width,
            height,
            shown,
            resolution,
            orientation,
            label,
        } = upload;

        if width == 0 || height == 0 {
            return None;
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self
            .render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
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
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * BYTES_PER_PIXEL as u32),
                rows_per_image: Some(height),
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
            size: shown,
            orientation,
            resolution,
            render_state: self.render_state.clone(),
        })
    }
}

/// Describes one of an image's surfaces as an upload.
fn describe<'a>(image: &'a DecodedImage, surface: &'a Surface, resolution: f32) -> Upload<'a> {
    let stored = Vec2::new(image.width() as f32, image.height() as f32);

    Upload {
        pixels: &surface.pixels,
        width: surface.width,
        height: surface.height,
        // The size the image is shown at does not depend on how much of it has
        // been uploaded, so nothing moves when the rest arrives.
        shown: crate::view::texture::displayed_size(stored, image.orientation),
        resolution,
        orientation: image.orientation,
        label: image.file_name(),
    }
}

#[cfg(test)]
mod tests {
    use super::is_short;

    /// A 6000 pixel wide image uploaded at 2560, shown fitted at 2560 points.
    #[test]
    fn a_reduced_copy_is_enough_at_the_size_it_was_made_for() {
        assert!(!is_short(2560.0 / 6000.0, 6000.0, 2560.0));
    }

    #[test]
    fn zooming_past_the_reduced_copy_asks_for_the_image_itself() {
        assert!(is_short(2560.0 / 6000.0, 6000.0, 4000.0));
    }

    #[test]
    fn a_rounding_error_does_not_ask_for_a_re_upload_every_frame() {
        assert!(!is_short(2560.0 / 6000.0, 6000.0, 2561.0));
    }

    #[test]
    fn a_full_resolution_texture_is_never_short() {
        assert!(!is_short(1.0, 6000.0, 60_000.0));
    }

    /// A camera thumbnail standing in for the image is short at any size worth
    /// looking at, so the real one is asked for as soon as it exists.
    #[test]
    fn a_thumbnail_is_short_as_soon_as_it_is_shown() {
        assert!(is_short(160.0 / 6000.0, 6000.0, 800.0));
    }
}
