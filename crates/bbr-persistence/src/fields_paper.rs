//! Schema-v4 space paper fields: the curated tint intensity and the static
//! grain flag (DESIGN §11.3, v2 D6/D7), persisted beside the space's tint
//! and glyph identity.

/// A curated space tint intensity (schema v4).
///
/// Closed identity data mirroring the portable space-appearance contract's
/// `TintIntensity`; the concrete wash strength per step is a presentation
/// concern.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TintIntensity {
    Subtle,
    #[default]
    Standard,
    Rich,
}

impl TintIntensity {
    /// All steps in stable picker order.
    pub const ALL: [TintIntensity; 3] = [Self::Subtle, Self::Standard, Self::Rich];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Standard => "standard",
            Self::Rich => "rich",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "subtle" => Some(Self::Subtle),
            "standard" => Some(Self::Standard),
            "rich" => Some(Self::Rich),
            _ => None,
        }
    }
}

/// A space's paper: tint intensity plus the static grain flag (schema v4).
/// The default is the deterministic `standard`, grain off.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpacePaper {
    pub intensity: TintIntensity,
    pub grain: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intensity_names_round_trip_and_reject_unknown() {
        for intensity in TintIntensity::ALL {
            assert_eq!(TintIntensity::parse(intensity.as_str()), Some(intensity));
        }
        assert_eq!(TintIntensity::parse("vivid"), None);
        assert_eq!(
            SpacePaper::default(),
            SpacePaper {
                intensity: TintIntensity::Standard,
                grain: false
            }
        );
    }
}
