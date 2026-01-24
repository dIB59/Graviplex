//! Texture atlas with automatic bin-packing for efficient sprite batching.
//!
//! This module provides:
//! - [`AtlasBuilder`] - Accumulates images and packs them into an atlas
//! - [`TextureAtlas`] - GPU texture with named region lookups
//! - [`AtlasRegion`] - UV coordinates and size for a sprite in the atlas
//!
//! # Example
//!
//! ```ignore
//! let atlas = AtlasBuilder::new()
//!     .add_image("player", "assets/player.png")?
//!     .add_image("enemy", "assets/enemy.png")?
//!     .add_image("bullet", "assets/bullet.png")?
//!     .build(&gpu, 2048)?;
//!
//! let region = atlas.get("player").unwrap();
//! ```

use std::collections::HashMap;
use std::path::Path;

/// A region within a texture atlas, containing UV coordinates and pixel dimensions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtlasRegion {
    /// UV coordinates: (u_min, v_min, u_max, v_max), normalized 0-1.
    pub uv: [f32; 4],
    /// Original pixel dimensions of the sprite.
    pub width: u32,
    pub height: u32,
}

impl AtlasRegion {
    /// Get the UV rect as an array for shader use.
    pub fn uv_rect(&self) -> [f32; 4] {
        self.uv
    }

    /// Get the size as a float array for convenience.
    pub fn size_f32(&self) -> [f32; 2] {
        [self.width as f32, self.height as f32]
    }
}

/// A GPU texture atlas with named region lookups.
///
/// Created by [`AtlasBuilder::build`] or [`Graphics::build_atlas`].
pub struct TextureAtlas {
    /// The GPU texture containing all packed sprites.
    #[allow(dead_code)]
    pub(crate) texture: wgpu::Texture,
    /// View into the texture for binding.
    #[allow(dead_code)]
    pub(crate) view: wgpu::TextureView,
    /// Sampler for texture filtering.
    #[allow(dead_code)]
    pub(crate) sampler: wgpu::Sampler,
    /// Bind group for the texture + sampler.
    pub(crate) bind_group: wgpu::BindGroup,
    /// Bind group layout (for pipeline creation).
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    /// Named regions within the atlas.
    regions: HashMap<String, AtlasRegion>,
    /// Atlas dimensions.
    pub width: u32,
    pub height: u32,
}

impl TextureAtlas {
    /// Get a region by name.
    pub fn get(&self, name: &str) -> Option<&AtlasRegion> {
        self.regions.get(name)
    }

    /// Get all region names.
    pub fn region_names(&self) -> impl Iterator<Item = &String> {
        self.regions.keys()
    }

    /// Get the number of regions in the atlas.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Check if the atlas contains a region with the given name.
    pub fn contains(&self, name: &str) -> bool {
        self.regions.contains_key(name)
    }
}

/// Error type for atlas building operations.
#[derive(Debug)]
pub enum AtlasError {
    /// Failed to load an image file.
    ImageLoad(String),
    /// Images don't fit in the specified atlas size.
    PackingFailed { max_size: u32 },
    /// No images were added to the builder.
    Empty,
}

impl std::fmt::Display for AtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtlasError::ImageLoad(path) => write!(f, "Failed to load image: {}", path),
            AtlasError::PackingFailed { max_size } => {
                write!(f, "Images don't fit in {}x{} atlas", max_size, max_size)
            }
            AtlasError::Empty => write!(f, "No images added to atlas builder"),
        }
    }
}

impl std::error::Error for AtlasError {}

/// Builder for creating texture atlases with automatic bin-packing.
///
/// # Example
///
/// ```ignore
/// let atlas = AtlasBuilder::new()
///     .add_image("player", "assets/player.png")?
///     .add_image("enemy", "assets/enemy.png")?
///     .build(&gpu, 2048)?;
/// ```
#[cfg(feature = "textures")]
#[derive(Debug)]
pub struct AtlasBuilder {
    pub(crate) images: HashMap<String, image::RgbaImage>,
}

#[cfg(feature = "textures")]
impl AtlasBuilder {
    /// Create a new empty atlas builder.
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    /// Add an image from a file path.
    pub fn add_image<P: AsRef<Path>>(mut self, name: &str, path: P) -> Result<Self, AtlasError> {
        let img = image::open(path.as_ref())
            .map_err(|_| AtlasError::ImageLoad(path.as_ref().display().to_string()))?
            .to_rgba8();
        self.images.insert(name.to_string(), img);
        Ok(self)
    }

    /// Add a sprite sheet (horizontal strip) and slice it into individual frames.
    ///
    /// This is the simplest API for sprite sheets arranged in a single horizontal row.
    /// Frame dimensions are derived automatically from the image and frame count.
    ///
    /// Frames are named `{prefix}_0`, `{prefix}_1`, etc.
    ///
    /// # Arguments
    /// * `prefix` - Base name for the frames (e.g., "player_walk" -> "player_walk_0", "player_walk_1", ...)
    /// * `path` - Path to the sprite sheet image
    /// * `frame_count` - Number of frames in the sprite sheet
    ///
    /// # Example
    /// ```ignore
    /// AtlasBuilder::new()
    ///     // A 192x64 image with 3 frames -> each frame is 64x64
    ///     .add_sprite_sheet("player_walk", "assets/player_walk.png", 3)?
    /// ```
    pub fn add_sprite_sheet<P: AsRef<Path>>(
        mut self,
        prefix: &str,
        path: P,
        frame_count: u32,
    ) -> Result<Self, AtlasError> {
        if frame_count == 0 {
            log::warn!("add_sprite_sheet: skipping '{}' with zero frame_count", prefix);
            return Ok(self);
        }

        let img = image::open(path.as_ref())
            .map_err(|_| AtlasError::ImageLoad(path.as_ref().display().to_string()))?
            .to_rgba8();

        let (sheet_width, sheet_height) = img.dimensions();
        let frame_width = sheet_width / frame_count;
        let frame_height = sheet_height;

        for i in 0..frame_count {
            let x = i * frame_width;
            let frame = image::imageops::crop_imm(&img, x, 0, frame_width, frame_height).to_image();
            let name = format!("{}_{}", prefix, i);
            self.images.insert(name, frame);
        }

        log::debug!(
            "Loaded sprite sheet '{}': {}x{} -> {} frames of {}x{}",
            prefix,
            sheet_width,
            sheet_height,
            frame_count,
            frame_width,
            frame_height
        );

        Ok(self)
    }

    /// Add a sprite sheet arranged in a grid (multiple rows and columns).
    ///
    /// Frames are extracted in row-major order (left-to-right, top-to-bottom)
    /// and named `{prefix}_0`, `{prefix}_1`, etc.
    ///
    /// # Arguments
    /// * `prefix` - Base name for the frames
    /// * `path` - Path to the sprite sheet image
    /// * `cols` - Number of columns (frames per row)
    /// * `rows` - Number of rows
    ///
    /// # Example
    /// ```ignore
    /// AtlasBuilder::new()
    ///     // A 256x128 image with 4 columns and 2 rows -> 8 frames of 64x64 each
    ///     .add_sprite_sheet_grid("explosion", "assets/explosion.png", 4, 2)?
    /// ```
    pub fn add_sprite_sheet_grid<P: AsRef<Path>>(
        mut self,
        prefix: &str,
        path: P,
        cols: u32,
        rows: u32,
    ) -> Result<Self, AtlasError> {
        if cols == 0 || rows == 0 {
            log::warn!("add_sprite_sheet_grid: skipping '{}' with zero cols or rows", prefix);
            return Ok(self);
        }

        let img = image::open(path.as_ref())
            .map_err(|_| AtlasError::ImageLoad(path.as_ref().display().to_string()))?
            .to_rgba8();

        let (sheet_width, sheet_height) = img.dimensions();
        let frame_width = sheet_width / cols;
        let frame_height = sheet_height / rows;

        let mut frame_index = 0;
        for row in 0..rows {
            for col in 0..cols {
                let x = col * frame_width;
                let y = row * frame_height;

                let frame = image::imageops::crop_imm(&img, x, y, frame_width, frame_height).to_image();
                let name = format!("{}_{}", prefix, frame_index);
                self.images.insert(name, frame);
                frame_index += 1;
            }
        }

        log::debug!(
            "Loaded sprite sheet grid '{}': {}x{} -> {} frames of {}x{}",
            prefix,
            sheet_width,
            sheet_height,
            frame_index,
            frame_width,
            frame_height
        );

        Ok(self)
    }

    /// Add a sprite sheet grid with explicit frame count.
    ///
    /// Use this when the sprite sheet grid doesn't fill completely
    /// (e.g., last row is partial).
    ///
    /// # Arguments
    /// * `prefix` - Base name for the frames
    /// * `path` - Path to the sprite sheet image
    /// * `cols` - Number of columns (frames per row)  
    /// * `frame_count` - Total number of frames to extract
    ///
    /// # Example
    /// ```ignore
    /// AtlasBuilder::new()
    ///     // A 256x128 sheet with 4 cols, but only 7 frames (last row has 3)
    ///     .add_sprite_sheet_partial("run", "assets/run.png", 4, 7)?
    /// ```
    pub fn add_sprite_sheet_partial<P: AsRef<Path>>(
        mut self,
        prefix: &str,
        path: P,
        cols: u32,
        frame_count: u32,
    ) -> Result<Self, AtlasError> {
        if cols == 0 || frame_count == 0 {
            log::warn!("add_sprite_sheet_partial: skipping '{}' with zero cols or frame_count", prefix);
            return Ok(self);
        }

        let img = image::open(path.as_ref())
            .map_err(|_| AtlasError::ImageLoad(path.as_ref().display().to_string()))?
            .to_rgba8();

        let (sheet_width, sheet_height) = img.dimensions();
        let frame_width = sheet_width / cols;
        let rows = frame_count.div_ceil(cols); // ceiling division
        let frame_height = sheet_height / rows;

        for i in 0..frame_count {
            let col = i % cols;
            let row = i / cols;
            let x = col * frame_width;
            let y = row * frame_height;

            let frame = image::imageops::crop_imm(&img, x, y, frame_width, frame_height).to_image();
            let name = format!("{}_{}", prefix, i);
            self.images.insert(name, frame);
        }

        log::debug!(
            "Loaded sprite sheet partial '{}': {}x{} -> {} frames of {}x{}",
            prefix,
            sheet_width,
            sheet_height,
            frame_count,
            frame_width,
            frame_height
        );

        Ok(self)
    }

    /// Add an image from raw RGBA bytes.
    pub fn add_image_bytes(
        mut self,
        name: &str,
        bytes: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Self, AtlasError> {
        let img = image::RgbaImage::from_raw(width, height, bytes.to_vec())
            .ok_or_else(|| AtlasError::ImageLoad(name.to_string()))?;
        self.images.insert(name.to_string(), img);
        Ok(self)
    }

    /// Add an image from an already-loaded RgbaImage.
    /// 
    /// Note: Consider using `add_solid_color`, `add_gradient`, or `add_checkerboard`
    /// for procedural textures without depending on the `image` crate directly.
    pub fn add_rgba_image(mut self, name: &str, img: image::RgbaImage) -> Self {
        self.images.insert(name.to_string(), img);
        self
    }

    /// Add a solid color rectangle.
    ///
    /// # Example
    /// ```ignore
    /// AtlasBuilder::new()
    ///     .add_solid_color("white", 32, 32, [255, 255, 255, 255])
    /// ```
    pub fn add_solid_color(mut self, name: &str, width: u32, height: u32, color: [u8; 4]) -> Self {
        if width == 0 || height == 0 {
            log::warn!("add_solid_color: skipping '{}' with zero dimension ({}x{})", name, width, height);
            return self;
        }
        let mut img = image::RgbaImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba(color);
        }
        self.images.insert(name.to_string(), img);
        self
    }

    /// Add a horizontal gradient rectangle.
    ///
    /// # Example
    /// ```ignore
    /// AtlasBuilder::new()
    ///     .add_gradient("sky", 64, 64, [135, 206, 235, 255], [25, 25, 112, 255])
    /// ```
    pub fn add_gradient(
        mut self,
        name: &str,
        width: u32,
        height: u32,
        color_left: [u8; 4],
        color_right: [u8; 4],
    ) -> Self {
        if width == 0 || height == 0 {
            log::warn!("add_gradient: skipping '{}' with zero dimension ({}x{})", name, width, height);
            return self;
        }
        let mut img = image::RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let t = x as f32 / width as f32;
                let r = (color_left[0] as f32 * (1.0 - t) + color_right[0] as f32 * t) as u8;
                let g = (color_left[1] as f32 * (1.0 - t) + color_right[1] as f32 * t) as u8;
                let b = (color_left[2] as f32 * (1.0 - t) + color_right[2] as f32 * t) as u8;
                let a = (color_left[3] as f32 * (1.0 - t) + color_right[3] as f32 * t) as u8;
                img.put_pixel(x, y, image::Rgba([r, g, b, a]));
            }
        }
        self.images.insert(name.to_string(), img);
        self
    }

    /// Add a checkerboard pattern.
    ///
    /// # Example
    /// ```ignore
    /// AtlasBuilder::new()
    ///     .add_checkerboard("checker", 64, 64, 8, [255, 255, 255, 255], [0, 0, 0, 255])
    /// ```
    pub fn add_checkerboard(
        mut self,
        name: &str,
        width: u32,
        height: u32,
        cell_size: u32,
        color1: [u8; 4],
        color2: [u8; 4],
    ) -> Self {
        if width == 0 || height == 0 {
            log::warn!("add_checkerboard: skipping '{}' with zero dimension ({}x{})", name, width, height);
            return self;
        }
        if cell_size == 0 {
            log::warn!("add_checkerboard: skipping '{}' with zero cell_size", name);
            return self;
        }
        let mut img = image::RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let checker = ((x / cell_size) + (y / cell_size)).is_multiple_of(2);
                let color = if checker { color1 } else { color2 };
                img.put_pixel(x, y, image::Rgba(color));
            }
        }
        self.images.insert(name.to_string(), img);
        self
    }

    /// Add a filled circle with optional border.
    ///
    /// # Example
    /// ```ignore
    /// AtlasBuilder::new()
    ///     .add_circle("ball", 48, [255, 0, 0, 255], None)
    ///     .add_circle("ring", 48, [0, 255, 0, 255], Some(([0, 200, 0, 255], 4)))
    /// ```
    pub fn add_circle(
        mut self,
        name: &str,
        diameter: u32,
        fill_color: [u8; 4],
        border: Option<([u8; 4], u32)>,
    ) -> Self {
        if diameter == 0 {
            log::warn!("add_circle: skipping '{}' with zero diameter", name);
            return self;
        }
        let mut img = image::RgbaImage::new(diameter, diameter);
        let center = diameter as f32 / 2.0;
        let radius = center - 1.0;
        
        for y in 0..diameter {
            for x in 0..diameter {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist <= radius {
                    let color = if let Some((border_color, border_width)) = border {
                        if dist > radius - border_width as f32 {
                            border_color
                        } else {
                            fill_color
                        }
                    } else {
                        fill_color
                    };
                    img.put_pixel(x, y, image::Rgba(color));
                } else {
                    img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                }
            }
        }
        self.images.insert(name.to_string(), img);
        self
    }

    /// Add a ring (hollow circle).
    ///
    /// # Example
    /// ```ignore
    /// AtlasBuilder::new()
    ///     .add_ring("halo", 64, 8, [255, 255, 0, 255])
    /// ```
    pub fn add_ring(mut self, name: &str, diameter: u32, thickness: u32, color: [u8; 4]) -> Self {
        if diameter == 0 {
            log::warn!("add_ring: skipping '{}' with zero diameter", name);
            return self;
        }
        if thickness == 0 {
            log::warn!("add_ring: skipping '{}' with zero thickness", name);
            return self;
        }
        let mut img = image::RgbaImage::new(diameter, diameter);
        let center = diameter as f32 / 2.0;
        let outer = center - 1.0;
        let inner = outer - thickness as f32;
        
        for y in 0..diameter {
            for x in 0..diameter {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist <= outer && dist >= inner {
                    img.put_pixel(x, y, image::Rgba(color));
                } else {
                    img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                }
            }
        }
        self.images.insert(name.to_string(), img);
        self
    }

    /// Build the texture atlas, packing all images into a single GPU texture.
    ///
    /// `max_size` is the maximum dimension (width and height) of the atlas.
    /// Common values are 1024, 2048, or 4096.
    pub fn build(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        max_size: u32,
    ) -> Result<TextureAtlas, AtlasError> {
        if self.images.is_empty() {
            return Err(AtlasError::Empty);
        }

        use rectangle_pack::{
            contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace,
            RectToInsert, TargetBin,
        };

        // Prepare rectangles for packing (with 1px padding to avoid bleeding)
        let mut rects_to_place: GroupedRectsToPlace<String, ()> = GroupedRectsToPlace::new();
        for (name, img) in &self.images {
            rects_to_place.push_rect(
                name.clone(),
                None,
                RectToInsert::new(img.width() + 2, img.height() + 2, 1),
            );
        }

        // Target bin
        let mut target_bins = std::collections::BTreeMap::new();
        target_bins.insert(0, TargetBin::new(max_size, max_size, 1));

        // Pack rectangles
        let packed = pack_rects(
            &rects_to_place,
            &mut target_bins,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .map_err(|_| AtlasError::PackingFailed { max_size })?;

        // Find actual used dimensions
        let mut actual_width = 0u32;
        let mut actual_height = 0u32;
        for (_, loc) in packed.packed_locations().values() {
            actual_width = actual_width.max(loc.x() + loc.width());
            actual_height = actual_height.max(loc.y() + loc.height());
        }

        // Ensure packed content fits within the maximum atlas size before creating the texture.
        if actual_width > max_size || actual_height > max_size {
            return Err(AtlasError::PackingFailed { max_size });
        }
        // Round up to power of 2 for GPU efficiency
        let atlas_width = actual_width.next_power_of_two().min(max_size);
        let atlas_height = actual_height.next_power_of_two().min(max_size);

        // Create the atlas image with checked size computation to avoid overflow
        let atlas_bytes = (atlas_width as u64)
            .checked_mul(atlas_height as u64)
            .and_then(|pixels| pixels.checked_mul(4)) // RGBA: 4 bytes per pixel
            .and_then(|bytes| usize::try_from(bytes).ok())
            .ok_or(AtlasError::PackingFailed { max_size })?;
        let mut atlas_data = vec![0u8; atlas_bytes];
        let mut regions = HashMap::new();

        for (name, (_, loc)) in packed.packed_locations() {
            let img = &self.images[name];

            // Account for 1px padding
            let x = loc.x() + 1;
            let y = loc.y() + 1;
            let w = img.width();
            let h = img.height();

            // Copy image data to atlas
            for py in 0..h {
                for px in 0..w {
                    let src_pixel = img.get_pixel(px, py);
                    let dst_idx = ((y + py) * atlas_width + (x + px)) as usize * 4;
                    atlas_data[dst_idx] = src_pixel[0];
                    atlas_data[dst_idx + 1] = src_pixel[1];
                    atlas_data[dst_idx + 2] = src_pixel[2];
                    atlas_data[dst_idx + 3] = src_pixel[3];
                }
            }

            // Calculate UV coordinates (normalized 0-1)
            let u_min = x as f32 / atlas_width as f32;
            let v_min = y as f32 / atlas_height as f32;
            let u_max = (x + w) as f32 / atlas_width as f32;
            let v_max = (y + h) as f32 / atlas_height as f32;

            regions.insert(
                name.clone(),
                AtlasRegion {
                    uv: [u_min, v_min, u_max, v_max],
                    width: w,
                    height: h,
                },
            );
        }

        // Create GPU texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Sprite Atlas"),
            size: wgpu::Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload texture data
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas_width),
                rows_per_image: Some(atlas_height),
            },
            wgpu::Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sprite Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Atlas Bind Group Layout"),
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

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Atlas Bind Group"),
            layout: &bind_group_layout,
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

        Ok(TextureAtlas {
            texture,
            view,
            sampler,
            bind_group,
            bind_group_layout,
            regions,
            width: atlas_width,
            height: atlas_height,
        })
    }
}

#[cfg(feature = "textures")]
impl Default for AtlasBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // UNIT TESTS - AtlasRegion
    // =========================================================================

    #[test]
    fn test_atlas_region_uv_rect() {
        let region = AtlasRegion {
            uv: [0.0, 0.0, 0.5, 0.5],
            width: 32,
            height: 32,
        };

        assert_eq!(region.uv_rect(), [0.0, 0.0, 0.5, 0.5]);
    }

    #[test]
    fn test_atlas_region_size_f32() {
        let region = AtlasRegion {
            uv: [0.0, 0.0, 0.5, 0.5],
            width: 64,
            height: 128,
        };

        assert_eq!(region.size_f32(), [64.0, 128.0]);
    }

    #[test]
    fn test_atlas_region_various_sizes() {
        // Test edge case: 1x1 pixel region
        let tiny = AtlasRegion {
            uv: [0.0, 0.0, 0.001, 0.001],
            width: 1,
            height: 1,
        };
        assert_eq!(tiny.size_f32(), [1.0, 1.0]);

        // Test large region
        let large = AtlasRegion {
            uv: [0.0, 0.0, 1.0, 1.0],
            width: 2048,
            height: 2048,
        };
        assert_eq!(large.size_f32(), [2048.0, 2048.0]);
    }

    // =========================================================================
    // UNIT TESTS - AtlasError
    // =========================================================================

    #[test]
    fn test_atlas_error_display_image_load() {
        let err = AtlasError::ImageLoad("missing.png".to_string());
        assert_eq!(format!("{}", err), "Failed to load image: missing.png");
    }

    #[test]
    fn test_atlas_error_display_packing_failed() {
        let err = AtlasError::PackingFailed { max_size: 1024 };
        assert_eq!(format!("{}", err), "Images don't fit in 1024x1024 atlas");
    }

    #[test]
    fn test_atlas_error_display_empty() {
        let err = AtlasError::Empty;
        assert_eq!(format!("{}", err), "No images added to atlas builder");
    }

    #[test]
    fn test_atlas_error_is_error_trait() {
        // Verify AtlasError implements std::error::Error
        fn assert_error<E: std::error::Error>() {}
        assert_error::<AtlasError>();
    }

    // =========================================================================
    // UNIT TESTS - AtlasBuilder (feature-gated)
    // =========================================================================

    #[cfg(feature = "textures")]
    mod builder_tests {
        use super::*;

        #[test]
        fn test_atlas_builder_new() {
            let builder = AtlasBuilder::new();
            // Builder starts empty - verified by checking default impl
            let builder2 = AtlasBuilder::default();
            // Both should be equivalent empty builders
            assert!(builder.images.is_empty());
            assert!(builder2.images.is_empty());
        }

        #[test]
        fn test_atlas_builder_add_rgba_image() {
            let img = image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255]));
            let builder = AtlasBuilder::new().add_rgba_image("red_square", img);

            assert!(builder.images.contains_key("red_square"));
            assert_eq!(builder.images.len(), 1);
        }

        #[test]
        fn test_atlas_builder_add_multiple_images() {
            let red = image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255]));
            let green = image::RgbaImage::from_pixel(64, 64, image::Rgba([0, 255, 0, 255]));
            let blue = image::RgbaImage::from_pixel(16, 16, image::Rgba([0, 0, 255, 255]));

            let builder = AtlasBuilder::new()
                .add_rgba_image("red", red)
                .add_rgba_image("green", green)
                .add_rgba_image("blue", blue);

            assert_eq!(builder.images.len(), 3);
            assert!(builder.images.contains_key("red"));
            assert!(builder.images.contains_key("green"));
            assert!(builder.images.contains_key("blue"));
        }

        #[test]
        fn test_atlas_builder_add_image_bytes() {
            // Create 2x2 RGBA image data
            let bytes: Vec<u8> = vec![
                255, 0, 0, 255, // Red pixel
                0, 255, 0, 255, // Green pixel
                0, 0, 255, 255, // Blue pixel
                255, 255, 0, 255, // Yellow pixel
            ];

            let builder = AtlasBuilder::new()
                .add_image_bytes("test", &bytes, 2, 2)
                .unwrap();

            assert!(builder.images.contains_key("test"));
            let img = &builder.images["test"];
            assert_eq!(img.width(), 2);
            assert_eq!(img.height(), 2);
        }

        #[test]
        fn test_atlas_builder_add_image_bytes_invalid() {
            // Wrong size for 2x2 RGBA (should be 16 bytes, not 8)
            let bytes: Vec<u8> = vec![255, 0, 0, 255, 0, 255, 0, 255];

            let result = AtlasBuilder::new().add_image_bytes("invalid", &bytes, 2, 2);

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AtlasError::ImageLoad(_)));
        }

        #[test]
        fn test_atlas_builder_overwrite_image() {
            let red = image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255]));
            let blue = image::RgbaImage::from_pixel(64, 64, image::Rgba([0, 0, 255, 255]));

            let builder = AtlasBuilder::new()
                .add_rgba_image("sprite", red)
                .add_rgba_image("sprite", blue); // Overwrites

            assert_eq!(builder.images.len(), 1);
            let img = &builder.images["sprite"];
            assert_eq!(img.width(), 64); // Should be the blue (64x64) image
        }

        #[test]
        fn test_atlas_builder_add_image_missing_file() {
            let result = AtlasBuilder::new().add_image("missing", "nonexistent_file.png");

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AtlasError::ImageLoad(_)));
        }
    }

    // =========================================================================
    // REGRESSION TESTS - Ensure existing behavior is preserved
    // =========================================================================

    #[test]
    fn test_atlas_region_copy_clone() {
        let region = AtlasRegion {
            uv: [0.1, 0.2, 0.3, 0.4],
            width: 100,
            height: 200,
        };

        let copied = region;
        let cloned = region.clone();

        assert_eq!(copied.uv, region.uv);
        assert_eq!(cloned.width, region.width);
    }

    #[test]
    fn test_atlas_region_debug_impl() {
        let region = AtlasRegion {
            uv: [0.0, 0.0, 1.0, 1.0],
            width: 32,
            height: 32,
        };

        let debug_str = format!("{:?}", region);
        assert!(debug_str.contains("AtlasRegion"));
        assert!(debug_str.contains("32"));
    }

    #[test]
    fn test_atlas_region_partial_eq() {
        let region1 = AtlasRegion {
            uv: [0.0, 0.0, 0.5, 0.5],
            width: 32,
            height: 32,
        };

        let region2 = AtlasRegion {
            uv: [0.0, 0.0, 0.5, 0.5],
            width: 32,
            height: 32,
        };

        let region3 = AtlasRegion {
            uv: [0.1, 0.1, 0.5, 0.5],
            width: 32,
            height: 32,
        };

        assert_eq!(region1, region2);
        assert_ne!(region1, region3);
    }
}
