//! GPU textures.
//!
//! A GPU texture is an object that can be perceived as an array on the GPU. It holds several items
//! and is a dimensional object. Textures are often associated with _images_, even though their
//! usage in the graphics world can be much larger than that.
//!
//! Textures — [`Texture`] come in several flavors and the differences lie in:
//!
//! - The dimensionality: textures can be 1D, 2D, 3D, layered, cube maps, etc.
//! - The encoding: the _items_ inside textures are called _pixels_ or _texels_. Those can be
//!   encoded in several ways.
//! - The usage: some textures will be used as images, others will be used to pass arbitrary data
//!   around in shaders, etc.
//!
//! Whatever the flavor, textures have few even not no use outside of shaders. When a shader wants
//! to use a texture, it can do it directly, by accessing each pixel by their position inside the
//! texture (using a normalized coordinates system) or via the use of a [`Sampler`]. A [`Sampler`],
//! as the name implies, is an object that tells the GPU how fetching (sampling) a texture should
//! occur. Several options there too:
//!
//! - The GPU can fetch a pixel without sampler. It means that you have to pass the exact
//!   coordinates of the pixel you want to access. This is useful when you store arbitrary
//!   information, like UUID, velocities, etc.
//! - The GPU can fetch a pixel with a floating-point coordinates system. How that system works
//!   depends on the settings of [`Sampler`] you choose. For instance, you can always fetch the
//!   _nearest_ pixel to where you sample, or you can ask the GPU to perform a linear
//!   interpolation between all neighboring pixels, etc. [`Sampler`] allow way more than that, so
//!   feel free to read their documentation.
//!
//! # Creating a texture

use std::error;
use std::fmt;
use std::marker::PhantomData;

use crate::backend::texture::Texture as TextureBackend;
use crate::context::GraphicsContext;
use crate::depth_test::DepthComparison;
use crate::pixel::{Pixel, PixelFormat};

/// How to wrap texture coordinates while sampling textures?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Wrap {
  /// If textures coordinates lay outside of *[0;1]*, they will be clamped to either *0* or *1* for
  /// every components.
  ClampToEdge,
  /// Textures coordinates are repeated if they lay outside of *[0;1]*. Picture this as:
  ///
  /// ```ignore
  /// // given the frac function returning the fractional part of a floating number:
  /// coord_ith = frac(coord_ith); // always between [0;1]
  /// ```
  Repeat,
  /// Same as `Repeat` but it will alternatively repeat between *[0;1]* and *[1;0]*.
  MirroredRepeat,
}

/// Minification filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinFilter {
  /// Nearest interpolation.
  Nearest,
  /// Linear interpolation between surrounding pixels.
  Linear,
  /// This filter will select the nearest mipmap between two samples and will perform a nearest
  /// interpolation afterwards.
  NearestMipmapNearest,
  /// This filter will select the nearest mipmap between two samples and will perform a linear
  /// interpolation afterwards.
  NearestMipmapLinear,
  /// This filter will linearly interpolate between two mipmaps, which selected texels would have
  /// been interpolated with a nearest filter.
  LinearMipmapNearest,
  /// This filter will linearly interpolate between two mipmaps, which selected texels would have
  /// been linarily interpolated as well.
  LinearMipmapLinear,
}

/// Magnification filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MagFilter {
  /// Nearest interpolation.
  Nearest,
  /// Linear interpolation between surrounding pixels.
  Linear,
}

/// Reify a type into a [`Dim`].
pub trait Dimensionable {
  /// Size type of a dimension (used to caracterize dimensions’ areas).
  type Size: Copy;

  /// Offset type of a dimension (used to caracterize addition and subtraction of sizes, mostly).
  type Offset: Copy;

  /// Zero offset.
  const ZERO_OFFSET: Self::Offset;

  /// Dimension.
  fn dim() -> Dim;

  /// Width of the associated [`Dimensionable::Size`].
  fn width(size: Self::Size) -> u32;

  /// Height of the associated [`Dimensionable::Size`]. If it doesn’t have one, set it to 1.
  fn height(_: Self::Size) -> u32 {
    1
  }

  /// Depth of the associated [`Dimensionable::Size`]. If it doesn’t have one, set it to 1.
  fn depth(_: Self::Size) -> u32 {
    1
  }

  /// X offset.
  fn x_offset(offset: Self::Offset) -> u32;

  /// Y offset. If it doesn’t have one, set it to 0.
  fn y_offset(_: Self::Offset) -> u32 {
    1
  }

  /// Z offset. If it doesn’t have one, set it to 0.
  fn z_offset(_: Self::Offset) -> u32 {
    1
  }

  /// Amount of pixels this size represents.
  ///
  /// For 2D sizes, it represents the area; for 3D sizes, the volume; etc.
  /// For cubemaps, it represents the side length of the cube.
  fn count(size: Self::Size) -> usize;
}

/// Dimension of a texture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dim {
  /// 1D.
  Dim1,
  /// 2D.
  Dim2,
  /// 3D.
  Dim3,
  /// Cubemap (i.e. a cube defining 6 faces — akin to 4D).
  Cubemap,
  /// 1D array.
  Dim1Array,
  /// 2D array.
  Dim2Array,
}

impl fmt::Display for Dim {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Dim::Dim1 => f.write_str("1D"),
      Dim::Dim2 => f.write_str("2D"),
      Dim::Dim3 => f.write_str("3D"),
      Dim::Cubemap => f.write_str("cubemap"),
      Dim::Dim1Array => f.write_str("1D array"),
      Dim::Dim2Array => f.write_str("2D array"),
    }
  }
}

/// 1D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim1;

impl Dimensionable for Dim1 {
  type Offset = u32;
  type Size = u32;

  const ZERO_OFFSET: Self::Offset = 0;

  fn dim() -> Dim {
    Dim::Dim1
  }

  fn width(w: Self::Size) -> u32 {
    w
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off
  }

  fn count(size: Self::Size) -> usize {
    size as usize
  }
}

/// 2D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim2;

impl Dimensionable for Dim2 {
  type Offset = [u32; 2];
  type Size = [u32; 2];

  const ZERO_OFFSET: Self::Offset = [0, 0];

  fn dim() -> Dim {
    Dim::Dim2
  }

  fn width(size: Self::Size) -> u32 {
    size[0]
  }

  fn height(size: Self::Size) -> u32 {
    size[1]
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off[0]
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off[1]
  }

  fn count([width, height]: Self::Size) -> usize {
    width as usize * height as usize
  }
}

/// 3D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim3;

impl Dimensionable for Dim3 {
  type Offset = [u32; 3];
  type Size = [u32; 3];

  const ZERO_OFFSET: Self::Offset = [0, 0, 0];

  fn dim() -> Dim {
    Dim::Dim3
  }

  fn width(size: Self::Size) -> u32 {
    size[0]
  }

  fn height(size: Self::Size) -> u32 {
    size[1]
  }

  fn depth(size: Self::Size) -> u32 {
    size[2]
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off[0]
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off[1]
  }

  fn z_offset(off: Self::Offset) -> u32 {
    off[2]
  }

  fn count([width, height, depth]: Self::Size) -> usize {
    width as usize * height as usize * depth as usize
  }
}

/// Cubemap dimension.
#[derive(Clone, Copy, Debug)]
pub struct Cubemap;

impl Dimensionable for Cubemap {
  type Offset = ([u32; 2], CubeFace);
  type Size = u32;

  const ZERO_OFFSET: Self::Offset = ([0, 0], CubeFace::PositiveX);

  fn dim() -> Dim {
    Dim::Cubemap
  }

  fn width(s: Self::Size) -> u32 {
    s
  }

  fn height(s: Self::Size) -> u32 {
    s
  }

  fn depth(_: Self::Size) -> u32 {
    6
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off.0[0]
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off.0[1]
  }

  fn z_offset(off: Self::Offset) -> u32 {
    match off.1 {
      CubeFace::PositiveX => 0,
      CubeFace::NegativeX => 1,
      CubeFace::PositiveY => 2,
      CubeFace::NegativeY => 3,
      CubeFace::PositiveZ => 4,
      CubeFace::NegativeZ => 5,
    }
  }

  fn count(size: Self::Size) -> usize {
    let size = size as usize;
    size * size
  }
}

/// Faces of a cubemap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CubeFace {
  /// The +X face of the cube.
  PositiveX,
  /// The -X face of the cube.
  NegativeX,
  /// The +Y face of the cube.
  PositiveY,
  /// The -Y face of the cube.
  NegativeY,
  /// The +Z face of the cube.
  PositiveZ,
  /// The -Z face of the cube.
  NegativeZ,
}

/// 1D array dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim1Array;

impl Dimensionable for Dim1Array {
  type Offset = (u32, u32);
  type Size = (u32, u32);

  const ZERO_OFFSET: Self::Offset = (0, 0);

  fn dim() -> Dim {
    Dim::Dim1Array
  }

  fn width(size: Self::Size) -> u32 {
    size.0
  }

  fn height(size: Self::Size) -> u32 {
    size.1
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off.0
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off.1
  }

  fn count((width, layer): Self::Size) -> usize {
    width as usize * layer as usize
  }
}

/// 2D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim2Array;

impl Dimensionable for Dim2Array {
  type Offset = ([u32; 2], u32);
  type Size = ([u32; 2], u32);

  const ZERO_OFFSET: Self::Offset = ([0, 0], 0);

  fn dim() -> Dim {
    Dim::Dim2Array
  }

  fn width(size: Self::Size) -> u32 {
    size.0[0]
  }

  fn height(size: Self::Size) -> u32 {
    size.0[1]
  }

  fn depth(size: Self::Size) -> u32 {
    size.1
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off.0[0]
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off.0[1]
  }

  fn z_offset(off: Self::Offset) -> u32 {
    off.1
  }

  fn count(([width, height], layer): Self::Size) -> usize {
    width as usize * height as usize * layer as usize
  }
}

/// A `Sampler` object gives hint on how a `Texture` should be sampled.
#[derive(Clone, Copy, Debug)]
pub struct Sampler {
  /// How should we wrap around the *r* sampling coordinate?
  pub wrap_r: Wrap,
  /// How should we wrap around the *s* sampling coordinate?
  pub wrap_s: Wrap,
  /// How should we wrap around the *t* sampling coordinate?
  pub wrap_t: Wrap,
  /// Minification filter.
  pub min_filter: MinFilter,
  /// Magnification filter.
  pub mag_filter: MagFilter,
  /// For depth textures, should we perform depth comparison and if so, how?
  pub depth_comparison: Option<DepthComparison>,
}

/// Default value is as following:
impl Default for Sampler {
  fn default() -> Self {
    Sampler {
      wrap_r: Wrap::ClampToEdge,
      wrap_s: Wrap::ClampToEdge,
      wrap_t: Wrap::ClampToEdge,
      min_filter: MinFilter::NearestMipmapLinear,
      mag_filter: MagFilter::Linear,
      depth_comparison: None,
    }
  }
}

/// Whether mipmaps should be generated.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GenMipmaps {
  /// Mipmaps should be generated.
  ///
  /// Mipmaps are generated when creating textures but also when uploading texels, clearing, etc.
  Yes,
  /// Never generate mipmaps.
  No,
}

/// Errors that might happen when working with textures.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureError {
  /// A texture’s storage failed to be created.
  ///
  /// The carried [`String`] gives the reason of the failure.
  TextureStorageCreationFailed(String),
  /// Not enough pixel data provided for the given area asked.
  ///
  /// You must provide at least as many pixels as expected by the area in the texture you’re
  /// uploading to.
  NotEnoughPixels {
    /// Expected number of pixels in bytes.
    expected_bytes: usize,
    /// Provided number of pixels in bytes.
    provided_bytes: usize,
  },
  /// Unsupported pixel format.
  ///
  /// Sometimes, some hardware might not support a given pixel format (or the format exists on
  /// the interface side but doesn’t in the implementation). That error represents such a case.
  UnsupportedPixelFormat(PixelFormat),
  /// Cannot retrieve texels from a texture.
  ///
  /// That error might happen on some hardware implementations if the user tries to retrieve
  /// texels from a texture that doesn’t support getting its texels retrieved.
  CannotRetrieveTexels(String),
  /// Failed to upload texels.
  CannotUploadTexels(String),
}

impl TextureError {
  /// A texture’s storage failed to be created.
  pub fn texture_storage_creation_failed(reason: impl Into<String>) -> Self {
    TextureError::TextureStorageCreationFailed(reason.into())
  }

  /// Not enough pixel data provided for the given area asked.
  pub fn not_enough_pixels(expected_bytes: usize, provided_bytes: usize) -> Self {
    TextureError::NotEnoughPixels {
      expected_bytes,
      provided_bytes,
    }
  }

  /// Unsupported pixel format.
  pub fn unsupported_pixel_format(pf: PixelFormat) -> Self {
    TextureError::UnsupportedPixelFormat(pf)
  }

  /// Cannot retrieve texels from a texture.
  pub fn cannot_retrieve_texels(reason: impl Into<String>) -> Self {
    TextureError::CannotRetrieveTexels(reason.into())
  }

  /// Failed to upload texels.
  pub fn cannot_upload_texels(reason: impl Into<String>) -> Self {
    TextureError::CannotUploadTexels(reason.into())
  }
}

impl fmt::Display for TextureError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TextureError::TextureStorageCreationFailed(ref e) => {
        write!(f, "texture storage creation failed: {}", e)
      }

      TextureError::NotEnoughPixels {
        ref expected_bytes,
        ref provided_bytes,
      } => write!(
        f,
        "not enough texels provided: expected {} bytes, provided {} bytes",
        expected_bytes, provided_bytes
      ),

      TextureError::UnsupportedPixelFormat(ref fmt) => {
        write!(f, "unsupported pixel format: {:?}", fmt)
      }

      TextureError::CannotRetrieveTexels(ref e) => {
        write!(f, "cannot retrieve texture’s texels: {}", e)
      }

      TextureError::CannotUploadTexels(ref e) => {
        write!(f, "cannot upload texels to texture: {}", e)
      }
    }
  }
}

impl error::Error for TextureError {}

/// GPU textures.
pub struct Texture<B, D, P>
where
  B: ?Sized + TextureBackend<D, P>,
  D: Dimensionable,
  P: Pixel,
{
  pub(crate) repr: B::TextureRepr,
  size: D::Size,
  _phantom: PhantomData<*const P>,
}

impl<B, D, P> Texture<B, D, P>
where
  B: ?Sized + TextureBackend<D, P>,
  D: Dimensionable,
  P: Pixel,
{
  /// Create a new [`Texture`] by reserving space for texels.
  ///
  /// `size` is the wished size of the [`Texture`].
  ///
  /// `mipmaps` is the number of extra mipmaps to allocate with the texture. `0` means that the
  /// texture will only be made of a _base level_.
  ///
  /// `sampler` is a [`Sampler`] object that will be used when sampling the texture from inside a
  /// shader, for instance.
  ///
  /// # Notes
  ///
  /// Feel free to have a look at the documentation of [`GraphicsContext::new_texture_no_texels`] for a
  /// simpler interface.
  pub(crate) fn new_no_texels<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: Sampler,
  ) -> Result<Self, TextureError>
  where
    C: GraphicsContext<Backend = B>,
  {
    unsafe {
      ctx
        .backend()
        .new_texture(size, mipmaps, sampler)
        .map(|repr| Texture {
          repr,
          size,
          _phantom: PhantomData,
        })
    }
  }

  /// Create a new [`Texture`].
  ///
  /// `size` is the wished size of the [`Texture`].
  ///
  /// `mipmaps` is the number of extra mipmaps to allocate with the texture. `0` means that the
  /// texture will only be made of a _base level_.
  ///
  /// `sampler` is a [`Sampler`] object that will be used when sampling the texture from inside a
  /// shader, for instance.
  ///
  /// `gen_mipmaps` determines whether mipmaps should be generated automatically.
  ///
  /// `texels` is a slice of raw texels to put into the texture store.
  ///
  /// # Notes
  ///
  /// Feel free to have a look at the documentation of [`GraphicsContext::new_texture`] for a
  /// simpler interface.
  pub fn new<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: Sampler,
    gen_mipmaps: GenMipmaps,
    texels: &[P::Encoding],
  ) -> Result<Self, TextureError>
  where
    C: GraphicsContext<Backend = B>,
  {
    let mut tex = unsafe {
      ctx
        .backend()
        .new_texture(size, mipmaps, sampler)
        .map(|repr| Texture {
          repr,
          size,
          _phantom: PhantomData,
        })?
    };
    tex.upload(gen_mipmaps, texels)?;
    Ok(tex)
  }

  /// Create a new [`Texture`] with raw texels.
  ///
  /// `size` is the wished size of the [`Texture`].
  ///
  /// `mipmaps` is the number of extra mipmaps to allocate with the texture. `0` means that the
  /// texture will only be made of a _base level_.
  ///
  /// `sampler` is a [`Sampler`] object that will be used when sampling the texture from inside a
  /// shader, for instance.
  ///
  /// `gen_mipmaps` determines whether mipmaps should be generated automatically.
  ///
  /// `texels` is a slice of raw texels to put into the texture store.
  ///
  /// # Notes
  ///
  /// Feel free to have a look at the documentation of [`GraphicsContext::new_texture_raw`] for a
  /// simpler interface.
  pub fn new_raw<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: Sampler,
    gen_mipmaps: GenMipmaps,
    texels: &[P::RawEncoding],
  ) -> Result<Self, TextureError>
  where
    C: GraphicsContext<Backend = B>,
  {
    let mut tex = unsafe {
      ctx
        .backend()
        .new_texture(size, mipmaps, sampler)
        .map(|repr| Texture {
          repr,
          size,
          _phantom: PhantomData,
        })?
    };
    tex.upload_raw(gen_mipmaps, texels)?;
    Ok(tex)
  }

  /// Return the number of mipmaps.
  pub fn mipmaps(&self) -> usize {
    unsafe { B::mipmaps(&self.repr) }
  }

  /// Return the size of the texture.
  pub fn size(&self) -> D::Size {
    self.size
  }

  /// Resize the texture by providing a new size and texels by reusing its GPU resources.
  ///
  /// This function works similarly to [`Texture::new`] but instead of creating a brand new texture, reuses the texture
  /// resources on the GPU.
  pub fn resize(
    &mut self,
    size: D::Size,
    mipmaps: usize,
    gen_mipmaps: GenMipmaps,
    texels: &[P::Encoding],
  ) -> Result<(), TextureError> {
    self.size = size;
    unsafe { B::resize(&mut self.repr, size, mipmaps) }?;
    self.upload(gen_mipmaps, texels)
  }

  /// Resize the texture by providing a new size and raw texels by reusing its GPU resources.
  ///
  /// This function works similarly to [`Texture::new_raw`] but instead of creating a brand new texture, reuses the texture
  /// resources on the GPU.
  pub fn resize_raw(
    &mut self,
    size: D::Size,
    mipmaps: usize,
    gen_mipmaps: GenMipmaps,
    texels: &[P::RawEncoding],
  ) -> Result<(), TextureError> {
    self.size = size;
    unsafe { B::resize(&mut self.repr, size, mipmaps) }?;
    self.upload_raw(gen_mipmaps, texels)
  }

  /// Clear the texture with a single pixel value.
  ///
  /// This function will assign the input pixel value to all the pixels in the rectangle described
  /// by `size` and `offset` in the texture.
  pub fn clear_part(
    &mut self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    pixel: P::Encoding,
  ) -> Result<(), TextureError> {
    unsafe { B::clear_part(&mut self.repr, gen_mipmaps, offset, size, pixel) }
  }

  /// Clear the texture with a single pixel value.
  ///
  /// This function will assign the input pixel value to all the pixels in the texture.
  pub fn clear(&mut self, gen_mipmaps: GenMipmaps, pixel: P::Encoding) -> Result<(), TextureError> {
    unsafe { B::clear(&mut self.repr, gen_mipmaps, self.size, pixel) }
  }

  /// Upload pixels to a region of the texture described by the rectangle made with `size` and
  /// `offset`.
  pub fn upload_part(
    &mut self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::Encoding],
  ) -> Result<(), TextureError> {
    unsafe { B::upload_part(&mut self.repr, gen_mipmaps, offset, size, texels) }
  }

  /// Upload pixels to the whole texture.
  pub fn upload(
    &mut self,
    gen_mipmaps: GenMipmaps,
    texels: &[P::Encoding],
  ) -> Result<(), TextureError> {
    unsafe { B::upload(&mut self.repr, gen_mipmaps, self.size, texels) }
  }

  /// Upload raw data to a region of the texture described by the rectangle made with `size` and
  /// `offset`.
  pub fn upload_part_raw(
    &mut self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::RawEncoding],
  ) -> Result<(), TextureError> {
    unsafe { B::upload_part_raw(&mut self.repr, gen_mipmaps, offset, size, texels) }
  }

  /// Upload raw data to the whole texture.
  pub fn upload_raw(
    &mut self,
    gen_mipmaps: GenMipmaps,
    texels: &[P::RawEncoding],
  ) -> Result<(), TextureError> {
    unsafe { B::upload_raw(&mut self.repr, gen_mipmaps, self.size, texels) }
  }

  /// Get a copy of all the pixels from the texture.
  pub fn get_raw_texels(&self) -> Result<Vec<P::RawEncoding>, TextureError>
  where
    P::RawEncoding: Copy + Default,
  {
    unsafe { B::get_raw_texels(&self.repr, self.size) }
  }
}
