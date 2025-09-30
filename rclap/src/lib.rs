// Licensed under the MIT license
// (see LICENSE or <http://opensource.org/licenses/MIT>) All files in the project carrying such
//! > rclap is a Rust utility that helps you create command-line interfaces with the clap crate by reducing boilerplate code. It generates clap structures from a simple TOML configuration file, allowing you to define your application's command-line arguments, environment variables, and default values in one place.
//!
//! # How it works
//!
//! 1- Create a TOML File: Define your configuration settings in a simple TOML file, specifying the argument name, type, default value, and associated environment variable.
//!
//! ```toml
//! port = { type = "int", default = "8080", doc = "Server port number", env = "PORT" }
//! ip = {  default = "localhost", doc = "connection URL", env = "URL" }
//! ```
//!
//! 2- Apply the Macro: Use the #[config] macro on an empty struct in your Rust code. The macro reads the TOML file and generates the complete clap::Parser implementation for you.
//!
//! ```text
//! use clap::Parser;
//! use rclap::config;
//!
//! #[config]
//! struct MyConfig;
//!
//! ```
//!
//! 3- Parse and Use: Your application can then simply call MyConfig::parse() to handle all command-line and environment variable parsing.
//!
//! ```text
//!
//! fn main() {
//!     let config = MyConfig::parse();
//!     println!("Config: {:#?}", config);
//!     println!("{}", &config.port);
//! }
//! ```
//!
//! rclap prioritizes a hierarchical approach to configuration, allowing you to set the ip and port via either environment variables or command-line arguments. If neither is specified, the predefined default values will be used.
//!
//! For instance, you can use the command-line flags --ip and --port to pass values directly. This would generate a help message like the one below, which clearly shows the available options, their default values, and the corresponding environment variables.
//!
//! ```text
//!
//! Usage: example [OPTIONS]
//!
//! Options:
//!       --"ip" <ip>      connection URL [env: URL=] [default: localhost]
//!       --"port" <port>  Server port number [env: PORT=120] [default: 8080]
//!   -h, --help           Print help
//! ```
pub use rclap_derive::config;
