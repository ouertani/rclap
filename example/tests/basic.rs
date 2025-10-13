use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;
use rclap::config;
use serial_test::serial;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum MyEnum {
    A,
    B,
    C,
}
#[test]
#[serial]
fn test_basic_file() {
    #[config]
    struct MyConfig;
    let config = MyConfig::parse();
    assert_eq!(config.port, 8080);
    assert_eq!(config.ip, "localhost".to_string());
}
#[test]
#[serial]
fn test_basic_file_with_env() {
    unsafe {
        std::env::set_var("PORT", "5432");
        std::env::set_var("IP", "127.0.0.1");
    }
    #[config]
    struct MyConfig;
    let config = MyConfig::parse();
    assert_eq!(config.port, 5432);
    assert_eq!(config.ip, "127.0.0.1".to_string());
    unsafe {
        std::env::remove_var("PORT");
        std::env::remove_var("IP");
    }
}
#[test]
#[serial]
fn test_second_file() {
    #[config]
    struct MyConfig;
    #[config("second_config.toml")]
    struct MySecondConfig;
    let config = MyConfig::parse();
    assert_eq!(config.port, 8080);
    assert_eq!(config.ip, "localhost".to_string());
    let second_config = MySecondConfig::parse();
    assert_eq!(second_config.port, 9080);
    assert_eq!(second_config.id, 2);
}
#[test]
#[serial]
fn test_option() {
    #[config("option.toml")]
    struct MyConfig;

    let config = MyConfig::parse();
    assert_eq!(config.port, 9080);
    assert_eq!(config.op_with_default, Some(2));
    assert_eq!(config.op_without_default, None);
}
#[test]
#[serial]
fn test_multi_types() {
    #[config("multi_types.toml")]
    struct MyConfig;

    let config = MyConfig::parse();
    assert_eq!(config.small_int, 80);
    assert_eq!(config.int, 8000);
    assert_eq!(config.float, 90.8);
    assert!(config.boolean);
    assert_eq!(config.string, "string_value");
    assert_eq!(config.path, PathBuf::from("."));
}
#[test]
#[serial]
fn test_inner_types() {
    #[config("config_with_inner.toml")]
    struct MyConfig;

    let config = MyConfig::parse();
    assert_eq!(config.port, 8080);
    assert_eq!(config.url, "localhost".to_string());
    assert_eq!(config.redis.url, "redis://localhost:6379".to_string());
    assert_eq!(config.redis.pool_size, 10);
}
#[test]
#[serial]
fn test_url_not_provided() {
    #[config("config_not_provided.toml")]
    struct MyConfig;
    let config = MyConfig::try_parse();

    assert!(config.is_err());
    unsafe {
        std::env::set_var("URL", "0.0.0.0");
    }

    let config = MyConfig::try_parse();

    assert!(config.is_ok());
    unsafe {
        std::env::remove_var("URL");
    }
}
