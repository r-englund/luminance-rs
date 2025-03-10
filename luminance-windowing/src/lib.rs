//! # The windowing types for luminance
//!
//! This crate provides you with a set of common types you can use when implementing windowing
//! crates. Even though the crate is used in most [luminance] windowing backends.
//!
//! [luminance]: https://crates.io/crates/luminance

#![deny(missing_docs)]

/// Dimension metrics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowDim {
  /// Windowed mode.
  Windowed {
    /// Width of the window.
    width: u32,
    /// Height of the window.
    height: u32,
  },
  /// Fullscreen mode (using the primary monitor resolution, for instance).
  Fullscreen,
  /// Fullscreen mode with restricted viewport dimension..
  FullscreenRestricted {
    /// Width of the window.
    width: u32,
    /// Height of the window.
    height: u32,
  },
}

/// Different window options.
///
/// Feel free to look at the different methods available to tweak the options. You may want to start
/// with `default()`, though.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowOpt {
  /// Dimension of the window.
  pub dim: WindowDim,
  /// Number of samples for multisampling.
  ///
  /// `None` means no multisampling.
  pub num_samples: Option<u32>,
}

impl Default for WindowOpt {
  /// Defaults:
  ///
  /// - `dim`: set to WindowDim::Windowed { width: 960, 540 }`.
  /// - `num_samples` set to `None`.
  fn default() -> Self {
    WindowOpt {
      dim: WindowDim::Windowed {
        width: 960,
        height: 540,
      },
      num_samples: None,
    }
  }
}

impl WindowOpt {
  /// Set the dimension of the window.
  #[inline]
  pub fn set_dim(self, dim: WindowDim) -> Self {
    WindowOpt { dim, ..self }
  }

  /// Get the dimension of the window.
  #[inline]
  pub fn dim(&self) -> &WindowDim {
    &self.dim
  }

  /// Set the number of samples to use for multisampling.
  ///
  /// Pass `None` to disable multisampling.
  #[inline]
  pub fn set_num_samples<S>(self, num_samples: S) -> Self
  where
    S: Into<Option<u32>>,
  {
    WindowOpt {
      num_samples: num_samples.into(),
      ..self
    }
  }

  /// Get the number of samples to use in multisampling, if any.
  #[inline]
  pub fn num_samples(&self) -> &Option<u32> {
    &self.num_samples
  }
}
