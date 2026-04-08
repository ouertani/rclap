pub mod default_secret;
pub type StringSecret = default_secret::Secret<String>;
pub use default_secret::Secret;
