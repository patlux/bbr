//! The G1b Foundations pages (packet `component-gallery-mvp.md`): the
//! gallery's own presentation of the `bbr-ui` token API — swatch tables,
//! type rows, density tables, the §10 state vocabulary, and the §7 icon
//! catalog. Every value derives from the real `bbr-ui` tokens; nothing is
//! copied or reimplemented.
//!
//! Split by page family so every module stays well below the ~500-line
//! budget: `tokens` (materials/density/elevation/color), `paper` (the v2
//! veils and space paper on the Color page), `type_motion`
//! (typography/spacing-radius), `motion` (v2 durations, easing, moments and
//! the live D10 illustrations), and `states_icons`
//! (states/iconography/brand). The catalog consumes the re-exports below.

pub mod motion;
pub mod paper;
pub mod states_icons;
pub mod tokens;
pub mod type_motion;

pub use motion::render_motion;
pub use states_icons::{render_brand, render_iconography, render_states};
pub use tokens::{render_color, render_density, render_elevation, render_tokens};
pub use type_motion::{render_spacing_radius, render_typography};
