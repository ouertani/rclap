use rclap::config;

#[config(derives=[serde::Serialize, serde::Deserialize])]
struct MyConfig;
#[config(path="second_config.toml" ,derives=[serde::Serialize, serde::Deserialize])]
struct MySecondConfig;

fn main() {
    let config = MyConfig::parse();
    println!("Config: {:#?}", config);
    let map = config.s.iter_map();
    for (key, value) in &map {
        println!("Key: {}, Value: {}", key, value);
    }
    // println!("Config as iter: {:?}", config.iter_map());
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    clap::ValueEnum,
    serde::Serialize,
    serde::Deserialize,
)]
enum MyEnum {
    A,
    B,
    C,
}
