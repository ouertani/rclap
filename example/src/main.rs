use clap::Parser;
use rclap::config;

#[config]
struct MyConfig;
#[config("config2.toml")]
struct MyConfig2;
fn main() {
    let config = MyConfig::parse();
    println!("Config: {:#?}", config);
    println!("{}", &config.port);
    println!("{:?}", &config.ip);

    let config2 = MyConfig2::parse();
    println!("Config2: {:#?}", config2);
}
