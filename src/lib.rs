//! # studio-ui
//!
//! The shared look of the Studio family of desktop apps ("The Data IDE"): design tokens for a dark and a
//! light theme, the embedded fonts (Inter, JetBrains Mono) and icon font (Lucide), and hand-painted
//! [egui](https://github.com/emilk/egui) widgets that match the family's mockups pixel for pixel.
//!
//! ```no_run
//! # fn setup(ctx: &egui::Context) {
//! studio_ui::theme::install(ctx); // fonts + both styles, dark first
//! # }
//! ```
//!
//! Every widget reads its colors from [`theme::Theme`]; nothing here knows about any product's domain.

pub mod chrome;
pub mod icons;
pub mod theme;
pub mod widgets;

pub use theme::{DARK, LIGHT, Theme};

/// The egui this crate is built against; use it instead of your own `egui` dependency so the versions
/// can never drift apart.
pub use egui;
