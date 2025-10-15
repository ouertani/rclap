use rclap::config;

#[config]
struct MyConfig;

fn main() {
    let config = MyConfig::parse();
    println!("Config: {:#?}", config);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum MyEnum {
    A,
    B,
    C,
}
