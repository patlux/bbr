//! The curated space paper (v2 D6/D7): three tint intensity steps and a
//! static grain flag beside the space's theme token and identity.

/// A curated tint intensity step (v2 D7: three steps, no free strength).
///
/// The domain carries the stable step identity only; the concrete wash
/// strength per step is a presentation concern resolved by the shell's
/// token layer (every step holds the DESIGN D3 contrast bars).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TintIntensity {
    /// A lighter wash than the default.
    Subtle,
    /// The deterministic default wash.
    #[default]
    Standard,
    /// A stronger wash than the default.
    Rich,
}

impl TintIntensity {
    /// All steps in stable picker order.
    pub const ALL: [TintIntensity; 3] = [
        TintIntensity::Subtle,
        TintIntensity::Standard,
        TintIntensity::Rich,
    ];

    /// The canonical machine-readable name (snapshots, E2E, ids).
    pub fn name(self) -> &'static str {
        match self {
            TintIntensity::Subtle => "subtle",
            TintIntensity::Standard => "standard",
            TintIntensity::Rich => "rich",
        }
    }

    /// The step for a canonical name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|step| step.name() == name)
    }

    /// A short human-readable picker label.
    pub fn label(self) -> &'static str {
        match self {
            TintIntensity::Subtle => "Subtle",
            TintIntensity::Standard => "Standard",
            TintIntensity::Rich => "Rich",
        }
    }
}

/// A space's paper: the tint intensity step and whether the static grain
/// (v2 D6: never animated) is painted. Defaults: `standard`, grain off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SpacePaper {
    pub intensity: TintIntensity,
    pub grain: bool,
}
