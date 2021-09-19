//! Pipeline backend interface.
//!
//! This interface defines the low-level API pipelines must implement to be usable.

use crate::{
  backend::{
    framebuffer::Framebuffer as FramebufferBackend,
    shader::ShaderData,
    shading_gate::ShadingGate as ShadingGateBackend,
    texture::{Texture, TextureBase},
  },
  pipeline::{PipelineError, PipelineState},
  pixel::Pixel,
  texture::Dimensionable,
};

pub unsafe trait PipelineBase: ShadingGateBackend + TextureBase {
  type PipelineRepr;

  unsafe fn new_pipeline(&mut self) -> Result<Self::PipelineRepr, PipelineError>;
}

pub unsafe trait Pipeline<D>: PipelineBase + FramebufferBackend<D>
where
  D: Dimensionable,
{
  unsafe fn start_pipeline(
    &mut self,
    framebuffer: &Self::FramebufferRepr,
    pipeline_state: &PipelineState,
  );
}

pub unsafe trait PipelineTexture<D, P>: PipelineBase + Texture<D, P>
where
  D: Dimensionable,
  P: Pixel,
{
  type BoundTextureRepr;

  unsafe fn bind_texture(
    pipeline: &Self::PipelineRepr,
    texture: &Self::TextureRepr,
  ) -> Result<Self::BoundTextureRepr, PipelineError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn texture_binding(bound: &Self::BoundTextureRepr) -> u32;
}

pub unsafe trait PipelineShaderData<T>: PipelineBase + ShaderData<T> {
  type BoundShaderData;

  unsafe fn bind_shader_data(
    pipeline: &Self::PipelineRepr,
    shader_data: &Self::ShaderDataRepr,
  ) -> Result<Self::BoundShaderData, PipelineError>;

  unsafe fn shader_data_binding(bound: &Self::BoundShaderData) -> u32;
}
