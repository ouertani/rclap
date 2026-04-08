pub mod secret;
pub type StringSecret = secret::Secret<String>;
pub use secret::Secret;
