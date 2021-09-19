use crate::Backend;

pub use luminance::tess::{
  Deinterleaved, DeinterleavedData, Interleaved, Mode, TessError, TessIndexType, TessMapError,
  TessViewError, View,
};

pub type TessBuilder<'a, V, I = (), W = (), S = Interleaved> =
  luminance::tess::TessBuilder<'a, Backend, V, I, W, S>;
pub type Tess<V, I = (), W = (), S = Interleaved> = luminance::tess::Tess<Backend, V, I, W, S>;
pub type Vertices<'a, V, I, W, S, T> = luminance::tess::Vertices<'a, Backend, V, I, W, S, T>;
pub type VerticesMut<'a, V, I, W, S, T> = luminance::tess::VerticesMut<'a, Backend, V, I, W, S, T>;
pub type Indices<'a, V, I, W, S> = luminance::tess::Indices<'a, Backend, V, I, W, S>;
pub type IndicesMut<'a, V, I, W, S> = luminance::tess::IndicesMut<'a, Backend, V, I, W, S>;
pub type Instances<'a, V, I, W, S, T> = luminance::tess::Instances<'a, Backend, V, I, W, S, T>;
pub type InstancesMut<'a, V, I, W, S, T> =
  luminance::tess::InstancesMut<'a, Backend, V, I, W, S, T>;
pub type TessView<'a, V, I, W, S> = luminance::tess::TessView<'a, Backend, V, I, W, S>;
