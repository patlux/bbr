//! Blueprint D12 (DESIGN §18.4): the remembered client-certificate choices
//! — per profile and server, the certificate's fingerprint (and its name,
//! for Settings) or "send none".
//!
//! Like the site permissions, the choices live in a small plain file next
//! to the store (`client-certificates.txt`), one tab-separated line each:
//! `profile host fingerprint|- name`. Lines that do not parse are skipped;
//! writes replace the file atomically.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bbr_web::ProfileId;

/// A remembered answer for one server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateChoice {
    /// Identify with the certificate of this fingerprint (and name).
    Identity { fingerprint: String, name: String },
    /// Continue without a certificate.
    None,
}

/// Every remembered choice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertificateChoices {
    entries: BTreeMap<(u64, String), CertificateChoice>,
}

fn clean(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c == '\t' || c == '\n' || c == '\r' {
                ' '
            } else {
                c
            }
        })
        .collect()
}

fn is_fingerprint(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

impl CertificateChoices {
    pub fn get(&self, profile: ProfileId, host: &str) -> Option<&CertificateChoice> {
        self.entries.get(&(profile.0, host.to_string()))
    }

    pub fn set(&mut self, profile: ProfileId, host: &str, choice: CertificateChoice) {
        self.entries.insert((profile.0, host.to_string()), choice);
    }

    pub fn forget(&mut self, profile: ProfileId, host: &str) {
        self.entries.remove(&(profile.0, host.to_string()));
    }

    pub fn forget_profile(&mut self, profile: ProfileId) {
        self.entries.retain(|(p, _), _| *p != profile.0);
    }

    /// The servers of `profile` with a remembered choice, in order.
    pub fn hosts(&self, profile: ProfileId) -> Vec<(String, CertificateChoice)> {
        self.entries
            .iter()
            .filter(|((p, _), _)| *p == profile.0)
            .map(|((_, host), choice)| (host.clone(), choice.clone()))
            .collect()
    }

    pub fn parse(body: &str) -> CertificateChoices {
        let mut choices = CertificateChoices::default();
        for line in body.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            let [profile, host, fingerprint, name] = parts[..] else {
                continue;
            };
            let Ok(profile) = profile.parse::<u64>() else {
                continue;
            };
            if host.is_empty() || host.contains(char::is_whitespace) {
                continue;
            }
            let choice = match fingerprint {
                "-" => CertificateChoice::None,
                fp if is_fingerprint(fp) => CertificateChoice::Identity {
                    fingerprint: fp.to_ascii_lowercase(),
                    name: name.to_string(),
                },
                _ => continue,
            };
            choices.set(ProfileId(profile), host, choice);
        }
        choices
    }

    pub fn render(&self) -> String {
        self.entries
            .iter()
            .map(|((profile, host), choice)| match choice {
                CertificateChoice::Identity { fingerprint, name } => {
                    format!("{profile}\t{host}\t{fingerprint}\t{}\n", clean(name))
                }
                CertificateChoice::None => format!("{profile}\t{host}\t-\t\n"),
            })
            .collect()
    }
}

/// The choices file of a profile directory.
pub fn path_in(profile_dir: &Path) -> PathBuf {
    profile_dir.join("client-certificates.txt")
}

pub fn load(path: &Path) -> CertificateChoices {
    std::fs::read_to_string(path)
        .map(|body| CertificateChoices::parse(&body))
        .unwrap_or_default()
}

pub fn save(path: &Path, choices: &CertificateChoices) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension("txt.tmp");
    std::fs::write(&temporary, choices.render())?;
    std::fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FP: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn choices_round_trip_per_profile_and_host() {
        let mut choices = CertificateChoices::default();
        choices.set(
            ProfileId(1),
            "vpn.corp.example",
            CertificateChoice::Identity {
                fingerprint: FP.into(),
                name: "Patrick\tWozniak".into(),
            },
        );
        choices.set(ProfileId(2), "vpn.corp.example", CertificateChoice::None);
        let parsed = CertificateChoices::parse(&choices.render());
        assert_eq!(
            parsed.get(ProfileId(1), "vpn.corp.example"),
            Some(&CertificateChoice::Identity {
                fingerprint: FP.into(),
                name: "Patrick Wozniak".into()
            })
        );
        assert_eq!(
            parsed.get(ProfileId(2), "vpn.corp.example"),
            Some(&CertificateChoice::None)
        );
        assert_eq!(parsed.get(ProfileId(3), "vpn.corp.example"), None);
        let tolerant =
            CertificateChoices::parse("1\thost\tnothex\tName\nx\thost\t-\t\n1\tok.example\t-\t\n");
        assert_eq!(tolerant.hosts(ProfileId(1)).len(), 1);
    }
}
