use serde_derive::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub package: Package,
    pub dependencies: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub children: Option<Vec<String>>,
}

pub fn parse_cfg(dir: &str) -> Config {
    let cfg_content =
        fs::read_to_string(dir.to_owned() + "Tau.toml").expect("could not open config file");
    return toml::from_str(cfg_content.as_str()).expect("could not parse config file");
}
