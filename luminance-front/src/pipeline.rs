use crate::Backend;

pub use luminance::pipeline::{PipelineError, PipelineState, TextureBinding, Viewport};

pub type Pipeline<'a> = luminance::pipeline::Pipeline<'a, Backend>;
pub type PipelineGate<'a> = luminance::pipeline::PipelineGate<'a, Backend>;
pub type BoundTexture<'a, D, P> = luminance::pipeline::BoundTexture<'a, Backend, D, P>;
pub type Render<E> = luminance::pipeline::Render<E>;
