//! Token machinery for the serde shape tests: the wire-shape serializer
//! ([`ser`]) and the token-stream deserializer ([`de`]).

pub(crate) mod de;
pub(crate) mod ser;

pub(crate) use de::D;
pub(crate) use ser::{Tok, S};
