//! Shader backend interface.
//!
//! This interface defines the low-level API shaders must implement to be usable.

use crate::shader::{
  ProgramError, ShaderDataError, StageError, StageType, TessellationStages, Uniform, UniformType,
  UniformWarning, VertexAttribWarning,
};
use crate::vertex::Semantics;

pub unsafe trait Uniformable<S>
where
  S: ?Sized + Shader,
{
  unsafe fn ty() -> UniformType;

  unsafe fn update(self, program: &mut S::ProgramRepr, uniform: &Uniform<Self>);
}

pub unsafe trait Shader {
  type StageRepr;

  type ProgramRepr;

  type UniformBuilderRepr;

  unsafe fn new_stage(&mut self, ty: StageType, src: &str) -> Result<Self::StageRepr, StageError>;

  unsafe fn new_program(
    &mut self,
    vertex: &Self::StageRepr,
    tess: Option<TessellationStages<Self::StageRepr>>,
    geometry: Option<&Self::StageRepr>,
    fragment: &Self::StageRepr,
  ) -> Result<Self::ProgramRepr, ProgramError>;

  unsafe fn apply_semantics<Sem>(
    program: &mut Self::ProgramRepr,
  ) -> Result<Vec<VertexAttribWarning>, ProgramError>
  where
    Sem: Semantics;

  unsafe fn new_uniform_builder(
    program: &mut Self::ProgramRepr,
  ) -> Result<Self::UniformBuilderRepr, ProgramError>;

  unsafe fn ask_uniform<T>(
    uniform_builder: &mut Self::UniformBuilderRepr,
    name: &str,
  ) -> Result<Uniform<T>, UniformWarning>
  where
    T: Uniformable<Self>;

  unsafe fn unbound<T>(uniform_builder: &mut Self::UniformBuilderRepr) -> Uniform<T>
  where
    T: Uniformable<Self>;
}

/// Shader data backend.
pub unsafe trait ShaderData<T> {
  /// Representation of the data by the backend.
  type ShaderDataRepr;

  /// Build a new shader data from some values.
  unsafe fn new_shader_data(
    &mut self,
    values: impl Iterator<Item = T>,
  ) -> Result<Self::ShaderDataRepr, ShaderDataError>;

  /// Access an item at index `i`.
  ///
  /// Return `None` if `i` is out-of-bounds.
  unsafe fn get_shader_data(shader_data: &Self::ShaderDataRepr, i: usize) -> Option<T>;

  /// Set an item at index `i`.
  ///
  /// Return `None` if `i` is out-of-bounds.
  unsafe fn set_shader_data(shader_data: &mut Self::ShaderDataRepr, i: usize, x: T) -> Option<()>;

  /// Update several items at once starting at a given index.
  unsafe fn update_shader_data(
    shader_data: &mut Self::ShaderDataRepr,
    i: usize,
    values: impl Iterator<Item = T>,
  ) -> Result<(), ShaderDataError>;
}
