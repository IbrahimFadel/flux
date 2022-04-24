use serde_derive::Deserialize;
use serde_repr::Deserialize_repr;
use std::{collections::HashMap, fs};

#[derive(Deserialize, Debug)]
pub struct Config {
	pub package: Package,
	pub dependencies: Dependencies,
	pub compilation: CompilationSettings,
}

#[derive(Deserialize, Debug)]
pub struct CompilationSettings {
	pub optimization: OptimizationLevel,
}

#[derive(Deserialize_repr, Debug, PartialEq)]
#[repr(u8)]
pub enum OptimizationLevel {
	None,
	Some,
	Highest,
}

pub type Dependencies = HashMap<String, Dependency>;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Dependency {
	Simple(String),
	Detailed(DependencyDetail),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct DependencyDetail {
	pub version: Option<String>,
	pub path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Package {
	pub name: String,
	pub version: String,
	pub children: Option<Vec<String>>,
}

pub fn parse_cfg(dir: &str) -> Config {
	let cfg_content =
		fs::read_to_string(dir.to_owned() + "pi.toml").expect("could not open config file");
	return toml::from_str(cfg_content.as_str()).expect("could not parse config file");
}
