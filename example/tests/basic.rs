use clap::Parser;
use rclap::config;
use serial_test::serial;

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
