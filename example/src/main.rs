use clap::Parser;
use rclap::config;

#[config("second_config.toml")]
struct MyConfig;
fn main() {
    let config = MyConfig::parse();
    println!("Config2: {:#?}", config.redis.is_some());
}
