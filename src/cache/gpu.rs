//! GPU residency: uploading decoded pixels into textures and keeping a bounded
//! number of them alive.
//!
//! An upload is a straight copy of the RGBA8 buffer the decoder produced, so
//! the images the user is about to reach are already sitting in VRAM when the
//! frame that draws them starts.

use eframe::egui_wgpu::RenderState;
use eframe::wgpu;
use epaint::{TextureId, Vec2};

use crate::decoder::preview::Preview;
use crate::decoder::{DecodedImage, BYTES_PER_PIXEL};
use crate::metadata::Orientation;

use super::mipmap::{self, MipGenerator};
use super::residency::Residency;

/// A texture owned by the viewer, freed when dropped.
pub struct GpuTexture {
    pub id: TextureId,
    /// Size the image is shown at, which a quarter turn swaps.
    pub size: Vec2,
    /// What this texture costs the adapter, mip chain included.
    ///
    /// Counted because a texture count is not a memory bound: two hundred
    /// thumbnails and two hundred sixty-megapixel photographs are the same
    /// number and a thousandfold difference in what the card is holding.
    pub bytes: usize,
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
    /// Turns what is already on the card, without decoding anything again.
    ///
    /// A rotation is written to the sidecar and read back by the decoder, so
    /// the lasting answer arrives the next time this photograph is decoded.
    /// This is the same answer, applied now: a quarter turn is four corners
    /// handed to the rasteriser in a different order, and waiting three
    /// hundred milliseconds for a sixty-megapixel raw to come round again
    /// before the picture moves is not an answer at all.
    pub fn turn(&mut self, extra: Orientation) {
        self.orientation = self.orientation.then(extra);

        if extra.transposes() {
            self.size = Vec2::new(self.size.y, self.size.x);
        }
    }

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

impl super::residency::Resident for GpuTexture {
    fn byte_len(&self) -> usize {
        self.bytes
    }
}

/// Textures resident on the GPU, keyed by image index.
pub struct GpuCache {
    /// What is held, budgeted by size *and* by count.
    ///
    /// Both, because they bound different failures: the count keeps the number
    /// of live texture descriptors sane, and the bytes keep a folder of very
    /// large photographs from filling the adapter's memory. Whichever is
    /// reached first is the one that evicts. The map itself, and all of that
    /// arithmetic, is `residency` — shared with the RAM cache, which is the
    /// same thing measured differently.
    entries: Residency<GpuTexture>,
    render_state: RenderState,
    /// Shared by every upload; building it costs a pipeline compile, which is
    /// not something to do per image.
    mipmaps: MipGenerator,
}

impl GpuCache {
    /// `capacity` is the number of textures to keep resident, and
    /// `budget_bytes` what they may add up to.
    pub fn new(render_state: RenderState, capacity: usize, budget_bytes: usize) -> GpuCache {
        let mipmaps = MipGenerator::new(&render_state.device);

        GpuCache {
            entries: Residency::bounded(budget_bytes, capacity),
            render_state,
            mipmaps,
        }
    }

    /// What the resident textures add up to.
    pub fn resident_bytes(&self) -> usize {
        self.entries.resident_bytes()
    }

    pub fn budget_bytes(&self) -> usize {
        self.entries.budget_bytes()
    }

    /// Largest texture edge the adapter supports.
    pub fn max_texture_edge(&self) -> u32 {
        self.render_state.adapter.limits().max_texture_dimension_2d
    }

    pub fn get(&self, index: usize) -> Option<&GpuTexture> {
        self.entries.get(index)
    }

    /// Turns whatever is resident at `index`, if anything is.
    pub fn turn(&mut self, index: usize, extra: Orientation) {
        if let Some(texture) = self.entries.get_mut(index) {
            texture.turn(extra);
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        self.entries.contains(index)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.entries.capacity().unwrap_or(usize::MAX)
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.entries.set_capacity(capacity);
    }

    /// Uploads whatever resolution `image` holds and evicts the texture
    /// furthest from `cursor` if the cache is over capacity.
    pub fn upload(&mut self, index: usize, image: &DecodedImage, cursor: usize, total: usize) {
        let stored = Vec2::new(image.width() as f32, image.height() as f32);

        self.put(
            index,
            Upload {
                pixels: &image.surface.pixels,
                width: image.surface.width,
                height: image.surface.height,
                // The size the image is shown at does not depend on how much
                // of it has been uploaded, so nothing moves when the rest
                // arrives.
                shown: crate::metadata::orientation::shown(stored, image.orientation),
                resolution: image.resolution(),
                orientation: image.orientation,
                label: image.file_name(),
            },
            cursor,
            total,
        );
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
        let shown = crate::metadata::orientation::shown(full, preview.orientation);

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

        self.entries.insert(index, texture, cursor, total);
    }

    pub fn remove(&mut self, index: usize) {
        self.entries.remove(index);
    }

    /// Removes a texture whose image has left the collection, shifting the
    /// entries above it down.
    pub fn remove_shifting(&mut self, index: usize) {
        self.entries.remove_shifting(index);
    }

    /// Makes room for a photograph appearing at `index`.
    pub fn insert_shifting(&mut self, index: usize) {
        self.entries.insert_shifting(index);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Drops every texture whose index is not in `keep`.
    pub fn retain(&mut self, keep: impl Fn(usize) -> bool) {
        self.entries.retain(keep);
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

        // Photographs are nearly always drawn smaller than they are stored, so
        // every texture carries its own shrunken copies.
        let mip_level_count = mipmap::levels(width, height);

        let texture = self
            .render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
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

        self.mipmaps.generate(
            &self.render_state.device,
            &self.render_state.queue,
            &texture,
            mip_level_count,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let id = self
            .render_state
            .renderer
            .write()
            .register_native_texture_with_sampler_options(
                &self.render_state.device,
                &view,
                wgpu::SamplerDescriptor {
                    label: Some("avis image"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    // The whole point of the chain: blend between the two
                    // levels either side of the size being drawn.
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                },
            );

        Some(GpuTexture {
            id,
            size: shown,
            bytes: mipmap::byte_len(width, height),
            orientation,
            resolution,
            render_state: self.render_state.clone(),
        })
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
