//! Reference image formats. Captures stay PNG; a store may keep its references as
//! lossless WebP. Every conversion is proven exact by decoding it back, and a
//! reference is only ever read through the strict decoders below.
use anyhow::{bail, ensure, Context, Result};
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Storage format of a baseline store's reference images.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[default]
    Png,
    Webp,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "png" => Ok(Self::Png),
            "webp" => Ok(Self::Webp),
            _ => bail!("Unsupported reference format {value:?}; expected png or webp"),
        }
    }

    /// Encodes exact RGBA8 pixels and proves the bytes decode back to them.
    pub fn encode(self, image: &RgbaImage) -> Result<Vec<u8>> {
        let bytes = match self {
            Self::Png => {
                let mut bytes = Vec::new();
                image.write_to(
                    &mut std::io::Cursor::new(&mut bytes),
                    image::ImageFormat::Png,
                )?;
                bytes
            }
            Self::Webp => encode_webp_lossless(image)?,
        };
        let back = self.decode(&bytes)?;
        ensure!(
            back.dimensions() == image.dimensions() && back.as_raw() == image.as_raw(),
            "Encoded {} reference does not decode to identical RGBA8 pixels",
            self.extension()
        );
        Ok(bytes)
    }

    /// Strict decoding: 8-bit PNG, or a simple lossless WebP (`VP8L`) file.
    pub fn decode(self, bytes: &[u8]) -> Result<RgbaImage> {
        match self {
            Self::Png => decode_png(bytes),
            Self::Webp => decode_webp_lossless(bytes),
        }
    }

    /// Cheap structural check without decoding: the PNG signature, or the exact
    /// single-`VP8L` lossless WebP container. Used where pixels are not needed.
    pub fn check(self, bytes: &[u8]) -> Result<()> {
        match self {
            Self::Png => {
                ensure!(
                    bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
                    "Expected PNG signature"
                );
                Ok(())
            }
            Self::Webp => check_webp_lossless(bytes),
        }
    }

    pub fn read(self, path: &Path) -> Result<RgbaImage> {
        let bytes = std::fs::read(path).with_context(|| format!("Read {}", path.display()))?;
        self.decode(&bytes)
            .with_context(|| format!("Invalid {} reference: {}", self.extension(), path.display()))
    }
}

/// Library-derived identity of decoded pixels: dimensions and every RGBA8 sample.
/// Equal fingerprints mean identical images; it is never taken from a producer.
pub fn pixel_hash(image: &RgbaImage) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"gpui-visual-rgba8\0");
    hasher.update(image.width().to_le_bytes());
    hasher.update(image.height().to_le_bytes());
    hasher.update(image.as_raw());
    format!("{:x}", hasher.finalize())
}

fn decode_png(bytes: &[u8]) -> Result<RgbaImage> {
    let reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    ensure!(
        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "Expected PNG signature"
    );
    let image = reader.decode()?;
    // Screenshots use at most 8 bits per channel. Do not silently quantize a
    // higher-precision input and report unequal samples as an exact match.
    ensure!(
        matches!(
            image.color(),
            image::ColorType::L8
                | image::ColorType::La8
                | image::ColorType::Rgb8
                | image::ColorType::Rgba8
        ),
        "16-bit PNG is unsupported; expected at most 8-bit channels"
    );
    Ok(image.into_rgba8())
}

/// Only the exact container this library writes: `RIFF <n> WEBP VP8L <n> …`,
/// one lossless bitstream and nothing else. Lossy (`VP8 `), extended (`VP8X`),
/// alpha-plane, animated or trailing data is rejected before decoding.
fn decode_webp_lossless(bytes: &[u8]) -> Result<RgbaImage> {
    check_webp_lossless(bytes)?;
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::WebP)?;
    ensure!(
        matches!(
            image.color(),
            image::ColorType::Rgb8 | image::ColorType::Rgba8
        ),
        "Unexpected WebP color type"
    );
    Ok(image.into_rgba8())
}

fn check_webp_lossless(bytes: &[u8]) -> Result<()> {
    ensure!(
        bytes.len() >= 21 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "Expected a WebP file"
    );
    let riff = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    ensure!(riff + 8 == bytes.len(), "WebP RIFF size mismatch");
    ensure!(
        &bytes[12..16] == b"VP8L",
        "Only lossless WebP (VP8L) references are accepted"
    );
    let chunk = u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;
    ensure!(
        20 + chunk + (chunk & 1) == bytes.len(),
        "Lossless WebP must contain exactly one bitstream chunk"
    );
    Ok(())
}

/// libwebp lossless preset 8 (method 5, quality 90) with `exact` RGB under
/// transparent pixels; single-threaded per image (callers parallelize images).
/// Measured on 3,306 UI references: 22% of the capture PNG bytes, ~85 ms/image;
/// preset 9 saves 0.3 points at ~12x the time.
pub const WEBP_LOSSLESS_LEVEL: i32 = 8;

fn encode_webp_lossless(image: &RgbaImage) -> Result<Vec<u8>> {
    use libwebp_sys as webp;
    let (width, height) = image.dimensions();
    ensure!(
        width > 0 && height > 0 && width <= 16383 && height <= 16383,
        "Image dimensions exceed lossless WebP limits"
    );
    // SAFETY: every libwebp struct is initialized by its versioned init function
    // before use; the picture and memory writer are always freed; the pixel slice
    // outlives the import call, which copies it into the picture.
    unsafe {
        let mut config: webp::WebPConfig = std::mem::zeroed();
        ensure!(
            webp::WebPConfigInitInternal(
                &mut config,
                webp::WebPPreset::WEBP_PRESET_DEFAULT,
                75.0,
                webp::WEBP_ENCODER_ABI_VERSION as i32,
            ) != 0,
            "libwebp configuration version mismatch"
        );
        ensure!(
            webp::WebPConfigLosslessPreset(&mut config, WEBP_LOSSLESS_LEVEL) != 0,
            "libwebp lossless preset rejected"
        );
        config.lossless = 1;
        config.exact = 1;
        config.near_lossless = 100;
        config.thread_level = 0;
        ensure!(
            webp::WebPValidateConfig(&config) != 0,
            "Invalid libwebp configuration"
        );
        let mut picture: webp::WebPPicture = std::mem::zeroed();
        ensure!(
            webp::WebPPictureInitInternal(&mut picture, webp::WEBP_ENCODER_ABI_VERSION as i32) != 0,
            "libwebp picture version mismatch"
        );
        picture.use_argb = 1;
        picture.width = width as i32;
        picture.height = height as i32;
        let mut writer: webp::WebPMemoryWriter = std::mem::zeroed();
        webp::WebPMemoryWriterInit(&mut writer);
        let imported =
            webp::WebPPictureImportRGBA(&mut picture, image.as_raw().as_ptr(), (width * 4) as i32);
        let encoded = imported != 0 && {
            picture.writer = Some(webp::WebPMemoryWrite);
            picture.custom_ptr = (&mut writer as *mut webp::WebPMemoryWriter).cast();
            webp::WebPEncode(&config, &mut picture) != 0
        };
        let error = picture.error_code;
        webp::WebPPictureFree(&mut picture);
        let bytes = (encoded && !writer.mem.is_null())
            .then(|| std::slice::from_raw_parts(writer.mem, writer.size).to_vec());
        webp::WebPMemoryWriterClear(&mut writer);
        bytes.with_context(|| format!("libwebp lossless encoding failed: {error:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn ui_like(width: u32, height: u32, seed: u8) -> RgbaImage {
        RgbaImage::from_fn(width, height, |x, y| {
            if (x / 7 + y / 5) % 3 == 0 {
                Rgba([seed, 30, 40, 255])
            } else if x % 11 == 0 {
                Rgba([(x ^ y) as u8, seed.wrapping_mul(3), 200, 128])
            } else {
                Rgba([250, 250, 250, 255])
            }
        })
    }

    #[test]
    fn webp_round_trip_is_exact_deterministic_and_smaller_than_png() {
        let mut image = ui_like(97, 61, 9);
        // Transparent pixels keep their RGB (`exact`), which lossy/default modes discard.
        image.put_pixel(0, 0, Rgba([12, 34, 56, 0]));
        let webp = Format::Webp.encode(&image).unwrap();
        assert_eq!(Format::Webp.encode(&image).unwrap(), webp, "deterministic");
        assert_eq!(&webp[12..16], b"VP8L");
        let back = Format::Webp.decode(&webp).unwrap();
        assert_eq!(back.as_raw(), image.as_raw());
        assert_eq!(pixel_hash(&back), pixel_hash(&image));
        let png = Format::Png.encode(&image).unwrap();
        assert_eq!(Format::Png.decode(&png).unwrap().as_raw(), image.as_raw());
        assert!(webp.len() < png.len(), "{} >= {}", webp.len(), png.len());
        let opaque = RgbaImage::from_pixel(8, 3, Rgba([1, 2, 3, 255]));
        let back = Format::Webp
            .decode(&Format::Webp.encode(&opaque).unwrap())
            .unwrap();
        assert_eq!(
            back.as_raw(),
            opaque.as_raw(),
            "opaque images decode to RGBA8"
        );
    }

    #[test]
    fn fingerprint_covers_every_sample_and_the_dimensions() {
        let image = ui_like(20, 10, 1);
        let mut changed = image.clone();
        let pixel = *changed.get_pixel(19, 9);
        changed.put_pixel(19, 9, Rgba([pixel[0], pixel[1], pixel[2], pixel[3] ^ 1]));
        assert_ne!(pixel_hash(&image), pixel_hash(&changed));
        let transposed = RgbaImage::from_raw(10, 20, image.as_raw().clone()).unwrap();
        assert_ne!(pixel_hash(&image), pixel_hash(&transposed));
        assert_eq!(pixel_hash(&image).len(), 64);
    }

    #[test]
    fn lossy_extended_truncated_and_foreign_files_are_rejected() {
        let image = ui_like(16, 16, 3);
        let webp = Format::Webp.encode(&image).unwrap();
        let mut lossy = webp.clone();
        lossy[12..16].copy_from_slice(b"VP8 ");
        assert!(format!("{:#}", Format::Webp.decode(&lossy).unwrap_err()).contains("lossless"));
        let mut extended = webp.clone();
        extended[12..16].copy_from_slice(b"VP8X");
        assert!(Format::Webp.decode(&extended).is_err());
        assert!(Format::Webp.decode(&webp[..webp.len() - 2]).is_err());
        let mut trailing = webp.clone();
        trailing.extend_from_slice(b"ICCP\0\0\0\0");
        assert!(Format::Webp.decode(&trailing).is_err());
        let mut corrupt = webp.clone();
        let middle = corrupt.len() / 2;
        corrupt[middle] ^= 0xff;
        assert!(Format::Webp
            .decode(&corrupt)
            .map_or(true, |back| back.as_raw() != image.as_raw()));
        let png = Format::Png.encode(&image).unwrap();
        assert!(Format::Webp.decode(&png).is_err());
        assert!(Format::Png.decode(&webp).is_err());
        assert_eq!(Format::parse("webp").unwrap(), Format::Webp);
        assert!(Format::parse("jxl").is_err());
    }
}
