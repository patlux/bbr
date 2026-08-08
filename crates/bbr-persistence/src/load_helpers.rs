//! Shared load helpers: deterministic order/identity validation and
//! persisted-value parsing shared by every loader.

use std::collections::{HashMap, HashSet};

use crate::{validation, CorruptionIssue, GlyphName, PersistenceError, SpaceGlyph, SpaceTint};

pub(crate) fn require_order(
    owner: &str,
    expected: usize,
    actual: i64,
) -> Result<(), PersistenceError> {
    if actual == expected as i64 {
        Ok(())
    } else {
        Err(PersistenceError::Corruption(
            CorruptionIssue::InvalidOrder {
                owner: owner.to_owned(),
            },
        ))
    }
}

pub(crate) fn require_owner_order(
    table: &str,
    owner: u64,
    actual: i64,
    positions: &mut HashMap<u64, usize>,
) -> Result<(), PersistenceError> {
    let expected = positions.entry(owner).or_default();
    require_order(&format!("{table} owner {owner}"), *expected, actual)?;
    *expected += 1;
    Ok(())
}

pub(crate) fn require_id(
    table: &str,
    value: i64,
    identities: &mut HashSet<u64>,
) -> Result<u64, PersistenceError> {
    let id = persisted_id(table, value)?;
    if identities.insert(id) {
        Ok(id)
    } else {
        Err(PersistenceError::Corruption(
            CorruptionIssue::DuplicateIdentity { id },
        ))
    }
}

pub(crate) fn persisted_id(table: &str, value: i64) -> Result<u64, PersistenceError> {
    if validation::valid_persisted_id(value) {
        Ok(value as u64)
    } else {
        Err(invalid_value(table, format!("invalid identity {value}")))
    }
}

pub(crate) fn require_bool(table: &str, value: i64) -> Result<bool, PersistenceError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(invalid_value(table, format!("invalid boolean {value}"))),
    }
}

/// Parses a persisted space appearance pair (`tint`, `glyph_kind`,
/// `glyph`) into the closed identity types.
///
/// All three persisted values are closed identity data: an unknown tint,
/// an unknown glyph kind, or a glyph value outside its curated set is
/// corruption, never a partial fallback.
pub(crate) fn parse_appearance(
    tint: &str,
    glyph_kind: &str,
    glyph: &str,
    space_id: u64,
) -> Result<(SpaceTint, SpaceGlyph), PersistenceError> {
    let tint = SpaceTint::parse(tint).ok_or_else(|| {
        invalid_value(
            "spaces",
            format!("space {space_id} has unknown tint `{tint}`"),
        )
    })?;
    let glyph = match glyph_kind {
        "emoji" => {
            if !crate::fields::ALLOWED_EMOJI.contains(&glyph) {
                return Err(invalid_value(
                    "spaces",
                    format!("space {space_id} has an uncurated emoji glyph"),
                ));
            }
            SpaceGlyph::Emoji(glyph.to_owned())
        }
        "glyph" => {
            let name = GlyphName::parse(glyph).ok_or_else(|| {
                invalid_value(
                    "spaces",
                    format!("space {space_id} has unknown glyph name `{glyph}`"),
                )
            })?;
            SpaceGlyph::Glyph(name)
        }
        _ => {
            return Err(invalid_value(
                "spaces",
                format!("space {space_id} has unknown glyph kind `{glyph_kind}`"),
            ));
        }
    };
    Ok((tint, glyph))
}

pub(crate) fn invalid_value(table: &str, detail: String) -> PersistenceError {
    PersistenceError::Corruption(CorruptionIssue::InvalidRequiredValue {
        table: table.to_owned(),
        detail,
    })
}

pub(crate) fn invalid_closed(tab_id: u64, detail: &str) -> PersistenceError {
    PersistenceError::Corruption(CorruptionIssue::InvalidRecentlyClosed {
        tab_id,
        detail: detail.to_owned(),
    })
}
