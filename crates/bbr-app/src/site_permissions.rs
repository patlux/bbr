//! Blueprint D11 (DESIGN §11.5): the remembered site permissions — Allow or
//! Block per profile, origin and capability; everything else is Ask.
//!
//! The browser store is SQLite with a strict schema stamp and no
//! migrations, so the permissions live apart in a small plain file next to
//! it (`site-permissions.txt` in the profile directory), one tab-separated
//! line per decision: `profile origin capability allow|block`. Lines that do
//! not parse are skipped, so an older or newer file never blocks startup.
//! Writes replace the file atomically (temporary file, then rename).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bbr_web::permissions::PermissionCapability;
use bbr_web::ProfileId;

/// One permission's setting for a site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SitePolicy {
    Allow,
    #[default]
    Ask,
    Block,
}

impl SitePolicy {
    /// Allow / Ask / Block, in the popover's order.
    pub const ALL: [SitePolicy; 3] = [SitePolicy::Allow, SitePolicy::Ask, SitePolicy::Block];

    pub fn label(self) -> &'static str {
        match self {
            SitePolicy::Allow => "Allow",
            SitePolicy::Ask => "Ask",
            SitePolicy::Block => "Block",
        }
    }
}

/// The capabilities BBR can decide for a page on macOS: WebKit asks the
/// browser for camera and microphone only (location, notifications and the
/// clipboard have no WebKit request BBR can answer).
pub const DECIDABLE: [PermissionCapability; 2] = [
    PermissionCapability::Camera,
    PermissionCapability::Microphone,
];

/// The wire key of a capability.
pub fn capability_key(capability: PermissionCapability) -> &'static str {
    match capability {
        PermissionCapability::Camera => "camera",
        PermissionCapability::Microphone => "microphone",
        PermissionCapability::Location => "location",
        PermissionCapability::Notifications => "notifications",
        PermissionCapability::Clipboard => "clipboard",
    }
}

/// The capability's display name.
pub fn capability_label(capability: PermissionCapability) -> &'static str {
    match capability {
        PermissionCapability::Camera => "Camera",
        PermissionCapability::Microphone => "Microphone",
        PermissionCapability::Location => "Location",
        PermissionCapability::Notifications => "Notifications",
        PermissionCapability::Clipboard => "Clipboard",
    }
}

fn parse_capability(key: &str) -> Option<PermissionCapability> {
    PermissionCapability::ALL
        .into_iter()
        .find(|capability| capability_key(*capability) == key)
}

/// The origin of `url` (`scheme://host[:port]`), or `None` for anything
/// that is not http(s).
pub fn origin_of(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    let authority = authority.rsplit('@').next()?;
    if authority.is_empty() {
        return None;
    }
    Some(format!("{scheme}://{}", authority.to_ascii_lowercase()))
}

/// The host of an origin (`127.0.0.1`, `meet.google.com`).
pub fn origin_host(origin: &str) -> &str {
    let rest = origin.split_once("://").map_or(origin, |(_, rest)| rest);
    if let Some(bracketed) = rest.strip_prefix('[') {
        return bracketed.split(']').next().unwrap_or(rest);
    }
    rest.split(':').next().unwrap_or(rest)
}

type Key = (u64, String, &'static str);

/// Every remembered decision.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SitePermissions {
    entries: BTreeMap<Key, bool>,
}

impl SitePermissions {
    fn key(profile: ProfileId, origin: &str, capability: PermissionCapability) -> Key {
        (profile.0, origin.to_string(), capability_key(capability))
    }

    /// The setting for one capability of a site.
    pub fn policy(
        &self,
        profile: ProfileId,
        origin: &str,
        capability: PermissionCapability,
    ) -> SitePolicy {
        match self.entries.get(&Self::key(profile, origin, capability)) {
            Some(true) => SitePolicy::Allow,
            Some(false) => SitePolicy::Block,
            None => SitePolicy::Ask,
        }
    }

    /// Change one setting (Ask forgets the decision).
    pub fn set(
        &mut self,
        profile: ProfileId,
        origin: &str,
        capability: PermissionCapability,
        policy: SitePolicy,
    ) {
        let key = Self::key(profile, origin, capability);
        match policy {
            SitePolicy::Allow => {
                self.entries.insert(key, true);
            }
            SitePolicy::Block => {
                self.entries.insert(key, false);
            }
            SitePolicy::Ask => {
                self.entries.remove(&key);
            }
        }
    }

    /// Forget every decision of a site in a profile.
    pub fn forget_site(&mut self, profile: ProfileId, origin: &str) {
        self.entries
            .retain(|(p, o, _), _| !(*p == profile.0 && o == origin));
    }

    /// Forget every decision of a deleted profile.
    pub fn forget_profile(&mut self, profile: ProfileId) {
        self.entries.retain(|(p, _, _), _| *p != profile.0);
    }

    /// The sites of `profile` with at least one decision, in order.
    pub fn sites(&self, profile: ProfileId) -> Vec<String> {
        let mut sites: Vec<String> = self
            .entries
            .keys()
            .filter(|(p, _, _)| *p == profile.0)
            .map(|(_, origin, _)| origin.clone())
            .collect();
        sites.dedup();
        sites
    }

    /// Parse the file body.
    pub fn parse(body: &str) -> SitePermissions {
        let mut permissions = SitePermissions::default();
        for line in body.lines() {
            let mut parts = line.split('\t');
            let (Some(profile), Some(origin), Some(capability), Some(value), None) = (
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
            ) else {
                continue;
            };
            let (Ok(profile), Some(capability)) =
                (profile.parse::<u64>(), parse_capability(capability))
            else {
                continue;
            };
            if origin_of(origin).as_deref() != Some(origin) {
                continue;
            }
            let policy = match value {
                "allow" => SitePolicy::Allow,
                "block" => SitePolicy::Block,
                _ => continue,
            };
            permissions.set(ProfileId(profile), origin, capability, policy);
        }
        permissions
    }

    /// The file body.
    pub fn render(&self) -> String {
        self.entries
            .iter()
            .map(|((profile, origin, capability), allow)| {
                let value = if *allow { "allow" } else { "block" };
                format!("{profile}\t{origin}\t{capability}\t{value}\n")
            })
            .collect()
    }
}

/// The permissions file of a profile directory.
pub fn path_in(profile_dir: &Path) -> PathBuf {
    profile_dir.join("site-permissions.txt")
}

/// Load the remembered decisions (none when the file is missing).
pub fn load(path: &Path) -> SitePermissions {
    std::fs::read_to_string(path)
        .map(|body| SitePermissions::parse(&body))
        .unwrap_or_default()
}

/// Write the decisions atomically.
pub fn save(path: &Path, permissions: &SitePermissions) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension("txt.tmp");
    std::fs::write(&temporary, permissions.render())?;
    std::fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SITE: &str = "https://meet.example.com";

    #[test]
    fn decisions_are_per_profile_origin_and_capability() {
        let mut permissions = SitePermissions::default();
        permissions.set(
            ProfileId(1),
            SITE,
            PermissionCapability::Camera,
            SitePolicy::Allow,
        );
        permissions.set(
            ProfileId(1),
            SITE,
            PermissionCapability::Microphone,
            SitePolicy::Block,
        );
        assert_eq!(
            permissions.policy(ProfileId(1), SITE, PermissionCapability::Camera),
            SitePolicy::Allow
        );
        assert_eq!(
            permissions.policy(ProfileId(2), SITE, PermissionCapability::Camera),
            SitePolicy::Ask,
            "never across profiles"
        );
        assert_eq!(
            permissions.policy(
                ProfileId(1),
                "https://other.example.com",
                PermissionCapability::Camera
            ),
            SitePolicy::Ask
        );
        permissions.set(
            ProfileId(1),
            SITE,
            PermissionCapability::Camera,
            SitePolicy::Ask,
        );
        assert_eq!(
            permissions.policy(ProfileId(1), SITE, PermissionCapability::Camera),
            SitePolicy::Ask
        );
        assert_eq!(permissions.sites(ProfileId(1)), vec![SITE.to_string()]);
        permissions.forget_site(ProfileId(1), SITE);
        assert!(permissions.sites(ProfileId(1)).is_empty());
    }

    #[test]
    fn the_file_round_trips_and_skips_what_it_cannot_read() {
        let mut permissions = SitePermissions::default();
        permissions.set(
            ProfileId(1),
            SITE,
            PermissionCapability::Camera,
            SitePolicy::Allow,
        );
        permissions.set(
            ProfileId(3),
            "http://127.0.0.1:8080",
            PermissionCapability::Microphone,
            SitePolicy::Block,
        );
        assert_eq!(SitePermissions::parse(&permissions.render()), permissions);
        let tolerant = SitePermissions::parse(
            "1\thttps://a.example\tcamera\tallow\nx\thttps://b.example\tcamera\tallow\n1\tnot an origin\tcamera\tallow\n1\thttps://c.example\tteleport\tallow\n1\thttps://d.example\tcamera\tmaybe\n",
        );
        assert_eq!(
            tolerant.sites(ProfileId(1)),
            vec!["https://a.example".to_string()]
        );
    }

    #[test]
    fn origins_come_from_http_urls_only() {
        assert_eq!(
            origin_of("https://Meet.Example.com/abc?x#y").as_deref(),
            Some("https://meet.example.com")
        );
        assert_eq!(
            origin_of("http://127.0.0.1:8080/page").as_deref(),
            Some("http://127.0.0.1:8080")
        );
        assert_eq!(
            origin_of("https://user@host.example/").as_deref(),
            Some("https://host.example")
        );
        assert_eq!(origin_of("bbr://settings"), None);
        assert_eq!(origin_of("file:///tmp/a"), None);
        assert_eq!(origin_host("http://127.0.0.1:8080"), "127.0.0.1");
        assert_eq!(origin_host("https://[::1]:3"), "::1");
        assert_eq!(origin_host("https://meet.example.com"), "meet.example.com");
    }
}
