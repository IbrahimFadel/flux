use serde::Deserialize;
use toml::Value;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub package: Option<Package>,
    pub workspace: Option<Workspace>,
    pub build: Build,
    pub dependencies: Dependencies,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Workspace {
    pub packages: Vec<String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Build {
    pub opt_level: OptLevel,
    pub ty: BuildType,
}

#[derive(Deserialize, Debug, Clone)]
pub enum OptLevel {
    None,   // 0
    Low,    // 1
    Medium, // 2
    High,   // 3
}

impl Default for OptLevel {
    fn default() -> Self {
        Self::Low
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum BuildType {
    Debug,
    Release,
}

impl Default for BuildType {
    fn default() -> Self {
        Self::Debug
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Dependencies {
    pub local: Vec<String>,
}

pub fn parse_cfg(content: &str) -> Config {
    let value: Value = toml::from_str(&content).expect("could not parse config file");

    let mut cfg = Config::default();

    if let Some(package_settings) = value.get("package") {
        cfg.package = Some(Package::default());

        let name = package_settings
            .get("name")
            .unwrap_or_else(|| panic!("missing name field in package settings"))
            .as_str()
            .expect("expected name type to be a string");
        let version = package_settings
            .get("version")
            .unwrap_or_else(|| panic!("missing version field in package settings"))
            .as_str()
            .expect("expected version type to be a string");

        let package = cfg.package.as_mut().unwrap();
        package.name = name.to_string();
        package.version = version.to_string();
    }
    if let Some(workspace_settings) = value.get("workspace") {
        if cfg.package.is_some() {
            panic!("cannot define package settings and workspace settings in the same flux.toml (they are mutually exclusive)");
        }

        cfg.workspace = Some(Workspace::default());
        let packages = workspace_settings.get("packages").map_or_else(
            || panic!("expected packages list in workspace settings"),
            |packages| {
                let packages: Vec<_> = packages
                    .as_array()
                    .expect("expected packages setting to be an array")
                    .iter()
                    .map(|package| {
                        package
                            .as_str()
                            .expect("expected package to be a string")
                            .to_string()
                    })
                    .collect();
                packages
            },
        );

        cfg.workspace.as_mut().unwrap().packages = packages;
    }
    if let Some(build_settings) = value.get("build") {
        if let Some(opt_level) = build_settings.get("opt-level") {
            let level = match opt_level
                .as_integer()
                .expect("expected opt-level to be integer")
            {
                0 => OptLevel::None,
                1 => OptLevel::Low,
                2 => OptLevel::Medium,
                3 => OptLevel::High,
                l => panic!("invalid opt-level `{l}`"),
            };
            cfg.build.opt_level = level;
        }
        if let Some(build_type) = build_settings.get("type") {
            let ty = match build_type
                .as_str()
                .expect("expected build type to be string")
            {
                "debug" => BuildType::Debug,
                "release" => BuildType::Release,
                t => panic!("invalid build type `{t}`"),
            };
            cfg.build.ty = ty;
        }
    }
    if let Some(dependencies) = value.get("dependencies") {
        if let Some(local) = dependencies.get("local") {
            let local = local
                .as_array()
                .expect("expected local dependencies to be an array")
                .iter()
                .map(|dep| {
                    dep.as_str()
                        .expect("expected local dependency to be string")
                        .to_string()
                })
                .collect();
            cfg.dependencies.local = local;
        }
    }

    cfg
}
