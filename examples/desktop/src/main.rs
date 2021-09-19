mod platform;

use glfw::{Action, Context as _, Key, Modifiers, MouseButton, WindowEvent};
use luminance_examples::{Example, InputAction, LoopFeedback};
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use platform::DesktopPlatformServices;
use std::{iter, path::PathBuf, time::Instant};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CLIOpts {
  #[structopt(short, long)]
  /// Directory where to pick textures from.
  textures: Option<PathBuf>,

  #[structopt(short, long)]
  /// List available examples.
  list_examples: bool,

  /// Example to run.
  example: Option<String>,
}

/// Macro to declaratively add examples.
macro_rules! examples {
  (examples: $($ex_name:literal, $test_ident:ident),* , funtests: $($fun_name:literal $(if $fun_feature_gate:literal)?, $fun_ident:ident),* $(,)?) => {
    fn show_available_examples() {
      println!("available examples:");
      $( println!("  - {}", $ex_name); )*

      #[cfg(feature = "funtest")]
      {
        println!("\navailable functional tests:");
        $(
          print!("  - {}", $fun_name);
          $(
            #[cfg(feature = $fun_feature_gate)]
            print!(" (feature: {})", $fun_feature_gate);
          )?
          println!("");
        )*
      }
    }

    // create a function that will run an example based on its name
    fn pick_and_run_example(cli_opts: CLIOpts) {
      let example_name = cli_opts.example.as_ref().map(|n| n.as_str());
      match example_name {
        $(
          Some($ex_name) => {
            run_example::<luminance_examples::$test_ident::LocalExample>(cli_opts, $ex_name)
          }
        ),*

        $(
          #[cfg(all(feature = "funtest"$(, feature = $fun_feature_gate)?))]
          Some($fun_name) => {
            run_example::<luminance_examples::$fun_ident::LocalExample>(cli_opts, $fun_name)
          }
        ),*

        _ => {
          log::error!("no example found");
          show_available_examples();
        }
      }
    }
  }
}

// Run an example.
fn run_example<E>(cli_opts: CLIOpts, name: &str)
where
  E: Example,
{
  // Check the features so that we know what we need to load.
  let mut services = DesktopPlatformServices::new(cli_opts, E::features());

  // First thing first: we create a new surface to render to and get events from.
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let surface =
    GlfwSurface::new_gl33(name, WindowOpt::default().set_dim(dim)).expect("GLFW surface creation");
  let mut context = surface.context;
  let events = surface.events_rx;

  let example = E::bootstrap(&mut services, &mut context);
  let start_t = Instant::now();

  // render a dummy frame to pass a single action containing the initial framebuffer size; some examples will use a
  // default size that is not correct, and this will allow them to bootstrap correctly
  let (fb_w, fb_h) = context.window.get_framebuffer_size();
  let feedback = example.render_frame(
    0.,
    context.back_buffer().unwrap(),
    iter::once(InputAction::Resized {
      width: fb_w as _,
      height: fb_h as _,
    }),
    &mut context,
  );
  let mut example = match feedback {
    LoopFeedback::Exit => return,
    LoopFeedback::Continue(example) => example,
  };

  'app: loop {
    // handle events
    context.window.glfw.poll_events();
    let actions = glfw::flush_messages(&events).flat_map(|(_, event)| adapt_events(event));

    let elapsed = start_t.elapsed();
    let t = elapsed.as_secs() as f64 + (elapsed.subsec_millis() as f64 * 1e-3);
    let feedback = example.render_frame(
      t as _,
      context.back_buffer().unwrap(),
      actions,
      &mut context,
    );

    if let LoopFeedback::Continue(stepped) = feedback {
      example = stepped;
      context.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}

fn adapt_events(event: WindowEvent) -> Option<InputAction> {
  match event {
    WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
      Some(InputAction::Quit)
    }

    WindowEvent::Key(Key::Space, _, Action::Release, mods) => {
      if mods.is_empty() {
        Some(InputAction::MainToggle)
      } else if mods == Modifiers::Shift {
        Some(InputAction::AuxiliaryToggle)
      } else {
        None
      }
    }

    WindowEvent::Key(key, _, Action::Press, _) | WindowEvent::Key(key, _, Action::Repeat, _) => {
      log::debug!("key press: {:?}", key);
      match key {
        Key::A => Some(InputAction::Left),
        Key::D => Some(InputAction::Right),
        Key::W => Some(InputAction::Forward),
        Key::S => Some(InputAction::Backward),
        Key::F => Some(InputAction::Up),
        Key::R => Some(InputAction::Down),
        _ => None,
      }
    }

    WindowEvent::MouseButton(MouseButton::Button1, action, _) => match action {
      Action::Press => Some(InputAction::PrimaryPressed),
      Action::Release => Some(InputAction::PrimaryReleased),
      _ => None,
    },

    WindowEvent::CursorPos(x, y) => Some(InputAction::CursorMoved {
      x: x as _,
      y: y as _,
    }),

    WindowEvent::FramebufferSize(width, height) => Some(InputAction::Resized {
      width: width as _,
      height: height as _,
    }),

    WindowEvent::Scroll(_, amount) => Some(InputAction::VScroll {
      amount: amount as f32,
    }),

    _ => None,
  }
}

examples! {
  examples:
  "hello-world", hello_world,
  "render-state", render_state,
  "sliced-tess", sliced_tess,
  "shader-uniforms", shader_uniforms,
  "attributeless", attributeless,
  "texture", texture,
  "offscreen", offscreen,
  "shader-uniform-adapt", shader_uniform_adapt,
  "dynamic-uniform-interface", dynamic_uniform_interface,
  "vertex-instancing", vertex_instancing,
  "query-texture-texels", query_texture_texels,
  "displacement-map", displacement_map,
  "interactive-triangle", interactive_triangle,
  "query-info", query_info,
  "mrt", mrt,
  "skybox", skybox,
  "shader-data", shader_data,

  // functional tests
  funtests:
  "funtest-tess-no-data", funtest_tess_no_data,
  "funtest-gl33-f64-uniform" if "funtest-gl33-f64-uniform", funtest_gl33_f64_uniform,
  "funtest-scissor-test", funtest_scissor_test,
  "funtest-360-manually-drop-framebuffer", funtest_360_manually_drop_framebuffer,
  "funtest-flatten-slice", funtest_flatten_slice,
  "funtest-pixel-array-encoding", funtest_pixel_array_encoding,
  "funtest-483-indices-mut-corruption", funtest_483_indices_mut_corruption,
}

fn main() {
  env_logger::builder()
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .init();
  let cli_opts = CLIOpts::from_args();

  if cli_opts.list_examples {
    show_available_examples();
  } else {
    pick_and_run_example(cli_opts);
  }
}
