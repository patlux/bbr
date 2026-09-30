//! How the app answers an HTTP authentication request
//! ([`crate::WebEvent::HttpAuthRequested`]).
//!
//! The portable challenge contract ([`crate::auth_challenges`]) decides with
//! opaque credential handles and never sees a password. The engine is the
//! boundary where the handle becomes the real credential: the app resolves
//! its in-memory credential and passes it here, once, to the engine that
//! hands it to WebKit. The type is never serialized and its `Debug` shape is
//! redacted, so no log, snapshot or event can carry the typed values.

use std::fmt;

/// A user name and password for one HTTP authentication answer.
#[derive(Clone, PartialEq, Eq)]
pub struct HttpCredential {
    user: String,
    password: String,
}

impl HttpCredential {
    pub fn new(user: String, password: String) -> Self {
        Self { user, password }
    }

    /// The user name (engine boundary only).
    pub fn user(&self) -> &str {
        &self.user
    }

    /// The password (engine boundary only).
    pub fn password(&self) -> &str {
        &self.password
    }
}

impl fmt::Debug for HttpCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HttpCredential(redacted)")
    }
}

/// The engine-level answer to one HTTP authentication request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpAuthAnswer {
    /// Sign in with this credential.
    Credential(HttpCredential),
    /// Answer without a credential: the request continues and the server's
    /// own answer (its 401 page) loads. Cancel, a superseded request and a
    /// view that goes away all answer this way.
    WithoutCredential,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_shows_the_credential() {
        let answer =
            HttpAuthAnswer::Credential(HttpCredential::new("someone".into(), "hunter2".into()));
        let shown = format!("{answer:?}");
        assert!(
            !shown.contains("someone") && !shown.contains("hunter2"),
            "{shown}"
        );
    }
}
