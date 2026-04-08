// Licensed under the MIT license
// (see LICENSE or <http://opensource.org/licenses/MIT>) All files in the project carrying such
//! > rclap is a Rust utility that helps you create command-line interfaces with the clap crate by reducing boilerplate code. It generates clap structures from a simple TOML configuration file, allowing you to define your application's command-line arguments, environment variables, and default values in one place.
//!
//! # Overview
//!
//! rclap lets you define your application's configuration in a single TOML file. Each setting
//! can reference an environment variable for overrides, providing flexibility between development,
//! testing, and production environments.
//!
//! # Quick Start
//!
//! ## 1- Create a TOML File
//!
//! Define your configuration with environment variable references:
//!
//! ```toml
//! port = { type = "int", default = "8080", env = "PORT" }
//! ip = { default = "localhost", env = "IP" }
//! ```
//!
//! ## 2- Apply the Macro
//!
//! Use the `#[config]` macro on an empty struct in your Rust code:
//!
//! ```text
//! #[config]
//! struct MyConfig;
//! ```
//!
//! ## 3- Parse and Use
//!
//! Call `MyConfig::parse()` to handle all command-line and environment variable parsing:
//!
//! ```text
//! fn main() {
//!     let config = MyConfig::parse();
//!     println!("Config: {:#?}", config);
//!     println!("{}", &config.port);
//! }
//! ```
//!
//! # Basic Configuration
//!
//! rclap supports several scalar types, each with optional `type`, `default`, `doc`, and `env` settings:
//!
//! ```toml
//! small_int = { type = "int", default = "80" }
//! float_val = { type = "float", default = "90.80" }
//! boolean = { type = "bool", default = "true" }
//! string_text = { type = "string", default = "hello" }
//! path_value = { type = "path", default = "." }
//! ```
//!
//! Environment variables can override any default value at runtime.
//!
//! # Advanced Features
//!
//! ## Enum Support
//!
//! Define custom enums either inline or reference external Rust enums:
//!
//! ```toml
//! # Inline enum definition
//! color = { enum = "Color", variants = ["Red", "Green", "Blue"], default = "Red", env = "COLOR" }
//! ```
//!
//! The above generates a Rust enum equivalent to:
//!
//! ```text
//! enum Color {
//!     Red,
//!     Green,
//!     Blue,
//! }
//! ```
//!
//! For external enums, reference them by path:
//!
//! ```toml
//! log_level = { enum = "crate::LogLevel", default = "INFO", env = "LOG_LEVEL" }
//! ```
//!
//! ## Array Types
//!
//! Use bracket notation to specify array types:
//!
//! ```toml
//! colors = { type = "[char]", default = ['R','G','B'], env = "COLORS" }
//! digits = { type = "[int]", default = [1,2,3], env = "DIGITS" }
//! ```
//!
//! ## Optional Values
//!
//! Mark fields as optional with `optional = true`. These fields may be absent from the config:
//!
//! ```toml
//! optional_with_default = { type = "int", default = "42", optional = true, env = "OPTIONAL_INT" }
//! optional_no_default = { type = "string", optional = true, env = "OPTIONAL_STR" }
//! ```
//!
//! ## Nested Configuration
//!
//! Create inner configuration structures by defining a section for nested fields:
//!
//! ```toml
//! app = { type = "AppConfig", default = "{ ... }", env = "APP_CONFIG" }
//!
//! [database]
//! url = { default = "localhost:5432", env = "DB_URL" }
//! pool_size = { type = "int", default = "10", env = "DB_POOL_SIZE" }
//! ```
//!
//! The `[database]` section generates a `Database` struct that can hold additional configuration.
//!
//! ## iter_map() Method
//!
//! Convert all configuration fields into a `HashMap<String, String>` for easy iteration:
//!
//! ```text
//! let config = MyConfig::parse();
//! let map = config.s.iter_map();
//! for (key, value) in &map {
//!     println!("{} = {}", key, value);
//! }
//! ```
//!
//! This is useful for dynamic configuration inspection, logging all settings, or serializing
//! to different formats.
//!
//! # Example Output
//!
//! rclap generates clear help messages showing available options, their environment variable
//! names, and default values:
//!
//! ```text
//! Usage: example [OPTIONS]
//!
//! Options:
//!       --"ip" <ip>      connection URL [env: IP=] [default: localhost]
//!       --"port" <port>  Server port number [env: PORT=] [default: 8080]
//!       -h, --help           Print help
//! ```
//!
//! # Feature Flags
//!
//! Enable the `secret` feature to access secure wrapper types for passwords, tokens, and API keys.

pub use rclap_derive::config;
#[cfg(feature = "secret")]
pub mod secrecy;
#[cfg(feature = "secret")]
pub use secrecy::*;
