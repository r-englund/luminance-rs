//! This program shows how to use shader data to provide data storage in shader stages. That storage can be used to
//! implement a wide variety of effects and tecnhiques, including geometry instancing (by passing the matrices). This
//! example showcases such a situation.
//!
//! https://docs.rs/luminance

use std::f32::consts::PI;

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::UniformInterface;
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::{PipelineState, ShaderDataBinding},
  render_state::RenderState,
  shader::{Program, ShaderData, Uniform},
  tess::{Mode, Tess, View},
  texture::Dim2,
  Backend,
};

const VS: &str = include_str!("./shader-data-2d-pos-vs.glsl");
const FS: &str = include_str!("./simple-fs.glsl");

const SQUARE_VERTICES: [Vertex; 4] = [
  Vertex::new(
    VertexPosition::new([-0.01, -0.01]),
    VertexColor::new([1., 0.5, 0.5]),
  ),
  Vertex::new(
    VertexPosition::new([0.01, -0.01]),
    VertexColor::new([1., 0.5, 0.5]),
  ),
  Vertex::new(
    VertexPosition::new([0.01, 0.01]),
    VertexColor::new([1., 0.5, 0.5]),
  ),
  Vertex::new(
    VertexPosition::new([-0.01, 0.01]),
    VertexColor::new([1., 0.5, 0.5]),
  ),
];

// shader interface to pass the shader data to the shader program
#[derive(UniformInterface)]
pub struct ShaderInterface {
  #[uniform(name = "Positions")]
  positions: Uniform<ShaderDataBinding<[f32; 2]>>,
}

pub struct LocalExample {
  program: Program<Semantics, (), ShaderInterface>,
  square: Tess<Vertex>,
  shader_data: ShaderData<[f32; 2]>,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = context
      .new_shader_program()
      .from_strings(VS, None, None, FS)
      .expect("program")
      .ignore_warnings();

    let square = context
      .new_tess()
      .set_vertices(&SQUARE_VERTICES[..])
      .set_mode(Mode::TriangleFan)
      .build()
      .expect("square tess");

    let instances_pos = [[0., 0.]; 100];
    let shader_data = context.new_shader_data(instances_pos).expect("shader data");

    Self {
      program,
      square,
      shader_data,
    }
  }

  fn render_frame(
    mut self,
    time: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,
        _ => (),
      }
    }

    // update shader data
    let instances_pos = (0..100).map(|i| {
      let i = i as f32;
      let i = i * PI * 2. * 0.01 + time * 0.5;
      [i.cos() * 0.7, i.sin() * 0.7]
    });

    self
      .shader_data
      .update(0, instances_pos)
      .expect("update shader data");

    let program = &mut self.program;
    let square = &self.square;
    let shader_data = &mut self.shader_data;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          // bind the shader data so that we can pass it to the shader program
          let bound_shader_data = pipeline
            .bind_shader_data(shader_data)
            .expect("bound shader data");

          shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
            iface.set(&uni.positions, ShaderDataBinding::from(&bound_shader_data));
            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(square.inst_view(.., 100).expect("instanced view"))
            })
          })
        },
      )
      .assume();

    if render.is_ok() {
      LoopFeedback::Continue(self)
    } else {
      LoopFeedback::Exit
    }
  }
}
