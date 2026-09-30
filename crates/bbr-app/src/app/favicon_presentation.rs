//! App-owned favicon image mirror (task packet `real-favicon-pipeline.md`).
//!
//! The controller owns the one bounded per-profile, host-keyed favicon
//! record (`BrowserController::favicons`). This module is the presentation
//! half of the join: it wraps each acquired record's bytes in gpui's
//! [`Image`] asset type exactly once per record (the decode itself runs
//! inside gpui's asset pipeline, keyed by content hash — never per frame),
//! and the plain presentation DTOs ([`RowVisuals`], [`FavoriteTileSpec`],
//! [`FolderMemberVisual`]) carry the wrapped image so every rendered
//! favicon slot — tab rows, pinned rows, Favorites tiles, folder items —
//! derives from the same store record through the existing GPUI `img`
//! element.
//!
//! ## Format sniffing
//!
//! The engine emits raw image bytes with no content type, so the mirror
//! sniffs the declared format from the bytes' magic (the same formats gpui's
//! asset pipeline decodes: PNG, JPEG, GIF, WebP, ICO, BMP, TIFF). Unknown
//! bytes default to PNG — the decode then fails inside gpui and the tile's
//! monogram fallback renders, the acquisition contract's silent-fallback
//! rule. The mirror refresh runs when the controller's store version
//! changes (bumped on every insert/eviction), so the decoded mirror always
//! matches the store.

use std::collections::HashMap;
use std::sync::Arc;

use bbr_web::ProfileId;
use gpui::{Image, ImageFormat};

use super::BbrApp;

/// One profile's `host → decoded-image` mirror (the same `(profile, host)`
/// key the controller's bounded store uses).
pub type FaviconImageMirror = HashMap<(ProfileId, String), Arc<Image>>;

/// Sniff the gpui [`ImageFormat`] from the acquisition bytes' magic. The
/// engine emits raw bytes with no content type; unknown bytes default to
/// PNG so the gpui asset decode fails (silent monogram fallback) rather
/// than the mirror claiming a format it cannot verify.
pub fn image_format_for(bytes: &[u8]) -> ImageFormat {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        ImageFormat::Png
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        ImageFormat::Jpeg
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        ImageFormat::Gif
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        ImageFormat::Webp
    } else if bytes.len() >= 4 && bytes[0] == 0 && bytes[1] == 0 && bytes[2] == 1 && bytes[3] == 0 {
        ImageFormat::Ico
    } else if bytes.starts_with(b"BM") {
        ImageFormat::Bmp
    } else if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        ImageFormat::Tiff
    } else {
        ImageFormat::Png
    }
}

impl BbrApp {
    /// Rebuild the decoded favicon image mirror when the controller's store
    /// version changed (bumped on every insert/eviction). Deterministic
    /// order: profile id, then the store's insertion order. Bytes are
    /// wrapped — never decoded here; the `Image` id hashes the content, so
    /// identical bytes share one gpui asset-cache entry.
    ///
    /// Called at the frame boundaries that derive presentation DTOs
    /// (`sidebar_model` via `u2_rail_data`, and the testkit snapshot), so
    /// every rendered favicon slot and every snapshot reads the mirror that
    /// matches the store version.
    pub(crate) fn refresh_favicon_mirror(&mut self) {
        let version = self.controller.read().favicon_version();
        if version == self.favicon_mirror_version.value() {
            return;
        }
        self.favicon_mirror_version.set(version);
        self.favicon_images.get().clear();
        let entries = self.controller.read().favicon_entries_all();
        for (profile, host, bytes) in entries {
            if bytes.is_empty() {
                continue;
            }
            self.favicon_images.get().insert(
                (profile, host),
                Arc::new(Image::from_bytes(image_format_for(&bytes), bytes)),
            );
        }
    }

    /// The decoded favicon image for `(profile, host)` from the mirror, if
    /// the owning profile's bounded store holds a record. Read-only: the
    /// mirror is refreshed at the frame boundary, never lazily here (this
    /// method is called from `&self` presentation derivations).
    pub(crate) fn favicon_image(&self, profile: ProfileId, host: &str) -> Option<Arc<Image>> {
        self.favicon_images
            .read()
            .get(&(profile, host.to_string()))
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_sniffing_covers_the_engine_emitted_shapes() {
        assert_eq!(image_format_for(b"\x89PNG\r\n\x1a\nrest"), ImageFormat::Png);
        assert_eq!(image_format_for(b"\xff\xd8\xff\xe0data"), ImageFormat::Jpeg);
        assert_eq!(image_format_for(b"GIF89a...."), ImageFormat::Gif);
        assert_eq!(image_format_for(b"RIFF....WEBPVP8 "), ImageFormat::Webp);
        assert_eq!(image_format_for(b"\x00\x00\x01\x00...."), ImageFormat::Ico);
        assert_eq!(image_format_for(b"BM...."), ImageFormat::Bmp);
        assert_eq!(image_format_for(b"II*\x00...."), ImageFormat::Tiff);
        // Unknown bytes default to PNG (the decode then fails silently).
        assert_eq!(image_format_for(b"not an image"), ImageFormat::Png);
        assert_eq!(image_format_for(b""), ImageFormat::Png);
    }
}
