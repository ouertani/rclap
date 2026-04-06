use rclap::config;

#[config(derives=[serde::Serialize, serde::Deserialize])]
struct MyConfig;
#[config(path="second_config.toml" ,derives=[serde::Serialize, serde::Deserialize])]
struct MySecondConfig;
#[config("config_with_secret.toml")]
struct MyConfigWithSecret;

fn main() {
    let config = MyConfig::parse();
    println!("Config: {:#?}", config);
    let map = config.s.iter_map();
    for (key, value) in &map {
        println!("Key: {}, Value: {}", key, value);
    }
    let secret_config = MyConfigWithSecret::parse();
    println!(
        "Secret Config: {:#?},  {:#?}",
        &secret_config.pwd,
        &secret_config.pwd.expose_secret()
    );
    println!("Secret Config: {:#?}", &secret_config.pwd_r,);
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
