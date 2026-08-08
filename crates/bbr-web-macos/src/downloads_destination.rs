//! Pure destination planning for download attempts: the L6 destination
//! policy (Explicit verbatim / Suggested folder resolution) and the
//! macOS-style uniquification rule. No AppKit, no filesystem, no `unsafe` —
//! the native side feeds [`uniquify_name`] a `fileExistsAtPath:` predicate.

use bbr_web::downloads::{DestinationDecision, StartRequest};

/// Where the attempt's file should land, handed to the native side for
/// resolution into an `NSURL`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationPlan {
    /// Use the path exactly as given — no folder, no uniquification (L6
    /// [`DestinationDecision::Explicit`]).
    Verbatim(String),
    /// Place `filename` in the engine's default download folder, uniquifying
    /// on collision (L6 [`DestinationDecision::Suggested`]).
    Suggest { filename: String },
}

/// The destination plan for a start request (pure; the L6 destination
/// policy, applied verbatim).
pub fn destination_plan(request: &StartRequest) -> DestinationPlan {
    match &request.destination {
        DestinationDecision::Explicit(path) => DestinationPlan::Verbatim(path.clone()),
        DestinationDecision::Suggested(name) => DestinationPlan::Suggest {
            filename: if name.is_empty() {
                url_filename(&request.url)
            } else {
                name.clone()
            },
        },
    }
}

/// Derive a suggested filename from a URL: the last non-empty path segment
/// (after the scheme+authority, with any query/fragment stripped); falls back
/// to `"download"` for URLs with no usable segment.
pub fn url_filename(url: &str) -> String {
    let trimmed = url.trim();
    let path = trimmed.split(['?', '#']).next().unwrap_or(trimmed);
    // Skip the scheme and authority; only the path portion may name a file.
    let after_authority = match path.find("://") {
        Some(idx) => {
            let rest = &path[idx + 3..];
            match rest.find('/') {
                Some(slash) => &rest[slash + 1..],
                None => "",
            }
        }
        None => path,
    };
    after_authority
        .rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or("download")
        .to_owned()
}

/// Split `name` into `(stem, extension)` at the last dot, treating a leading
/// dot (hidden files) as part of the stem.
fn split_extension(name: &str) -> (String, String) {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem.to_owned(), ext.to_owned()),
        _ => (name.to_owned(), String::new()),
    }
}

/// The macOS "name 2.ext" uniquification: the first name not reported by
/// `exists` wins. `exists` is injected so the pure rule is testable without
/// a filesystem; the native side feeds it `fileExistsAtPath:`.
pub fn uniquify_name(name: &str, exists: &dyn Fn(&str) -> bool) -> String {
    if !exists(name) {
        return name.to_owned();
    }
    let (stem, ext) = split_extension(name);
    for n in 2.. {
        let candidate = if ext.is_empty() {
            format!("{stem} {n}")
        } else {
            format!("{stem} {n}.{ext}")
        };
        if !exists(&candidate) {
            return candidate;
        }
    }
    unreachable!("u64 counter cannot be exhausted")
}

#[cfg(test)]
mod tests;
