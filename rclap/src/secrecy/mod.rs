//! Secret module - Provides secure handling of sensitive values.
//!
//! This module offers types for securely managing secrets like passwords, tokens, and API keys.
//! The `secrecy` feature must be enabled when using these types.
//!
//! # Types
//!
//! ## `Secret<S>`
//!
//! A generic wrapper that holds a sensitive value of type `S` (where `S: CloneableSecret`).
//! - Stores the value securely (not shown in plain text)
//! - `Debug` and `Display` implementations show `*` instead of the actual value
//! - Supports serialization/deserialization via serde
//!
//! ## `StringSecret`
//!
//! A specialized wrapper for `String` secrets.
//! - Similar behavior to `Secret<String>`
//! - Useful for passwords, tokens, or any string-based secrets
//!
//!
//! # Feature Flag
//!
//! These types are conditionally compiled when the `secrecy` Cargo feature is enabled.
//!

pub mod secret;
pub mod string_secret;
pub use secret::Secret;
pub use string_secret::StringSecret;
