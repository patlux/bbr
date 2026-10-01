//! Stable machine diagnostics. Classify typed causes, never parse display strings.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    InvalidArguments,
    InvalidInput,
    UnknownStory,
    SelectionUnsupported,
    SourceChanged,
    EnvironmentMismatch,
    ProducerFailed,
    EnvironmentProbeFailed,
    ProcessTimeout,
    HostGateUnfinished,
}
impl Code {
    pub fn error(self, message: impl Into<String>) -> Error {
        Error {
            code: self,
            message: message.into(),
        }
    }
    pub fn next_step(self) -> &'static str {
        match self {
            Self::InvalidArguments => "Read this command's --help; no automatic retry.",
            Self::InvalidInput => "Inspect the diagnostic and inputs; do not rewrite seals or references.",
            Self::UnknownStory => "Use list to discover exact story IDs, then request an explicit selection.",
            Self::SelectionUnsupported => "Integrate the story-selection protocol in the consumer; never fall back to a full capture silently.",
            Self::SourceChanged => "Capture a fresh run from the current sources using the authorized consumer route.",
            Self::EnvironmentMismatch => "Investigate environment drift; do not automatically adopt the environment.",
            Self::ProducerFailed | Self::EnvironmentProbeFailed => "Inspect producer/probe diagnostics; retain the failed run and fix the cause before a new capture.",
            Self::ProcessTimeout => "Inspect the timed-out command and cleanup result; do not blindly retry or increase deadlines.",
            Self::HostGateUnfinished => "Inspect the owner and recorded groups. Explicit verified recovery is required; never delete the lock automatically.",
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Error {
    pub code: Code,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Diagnostic {
    pub code: Code,
    pub message: String,
    pub next_step: String,
}
impl Diagnostic {
    pub fn from_error(error: &anyhow::Error) -> Self {
        let code = error
            .downcast_ref::<Error>()
            .map_or(Code::InvalidInput, |e| e.code);
        Self {
            code,
            message: format!("{error:#}"),
            next_step: code.next_step().into(),
        }
    }
}
