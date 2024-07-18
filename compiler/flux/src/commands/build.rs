use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::{diagnostics::DriverError, ExitStatus, INTERNER};

use flux_cfg::{Config, CFG_FILE_NAME};
use flux_diagnostics::{ice, IOError, SourceCache};
use flux_hir::{lower_package_bodies, lower_package_defs, PackageBodies, PackageId};
use flux_typesystem::TEnv;
use la_arena::{Arena, ArenaMap};
use lasso::ThreadedRodeo;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Path to root directory of the flux project
    ///
    /// Defaults to current directory
    #[arg(long)]
    root_path: Option<PathBuf>,

    /// Debug the CST
    ///
    /// Defaults to false.
    /// If true prints the CST
    #[arg(long)]
    debug_cst: bool,

    /// Debug the Item Tree
    ///
    /// Defaults to false.
    #[arg(long)]
    debug_item_tree: bool,

    /// Debug the Item Tree with Expression Bodies
    ///
    /// Defaults to false.
    #[arg(long)]
    debug_with_bodies: bool,
}

pub fn build(args: Args) -> ExitStatus {
    let project_root = args.root_path.unwrap_or(env::current_dir().unwrap_or_else(|err| ice(&format!("could not determine project root path, make sure you have the proper permissions for this directory: {:?}", err))));
    let debug_cst = args.debug_cst;
    let debug_item_tree = args.debug_item_tree;
    let debug_with_bodies = args.debug_with_bodies;
    tracing::info!(project_root =? project_root, "executing build command");

    let cfg = match get_config(&project_root) {
        Ok(cfg) => cfg,
        Err(diagnostic) => {
            diagnostic.report();
            return ExitStatus::Failure;
        }
    };

    let lowering_config = flux_hir::cfg::Config {
        debug_cst,
        debug_item_tree,
        debug_with_bodies,
    };

    let interner = INTERNER.get_or_init(|| {
        ThreadedRodeo::from_iter([
            "s64", "s32", "s16", "s8", "u64", "u32", "u16", "u8", "f64", "f32", "str", "bool",
            "true", "false",
        ])
    });
    let mut packages = Arena::new();
    let mut package_bodies = ArenaMap::new();
    // let mut package_exprs = ArenaMap::new();
    // let mut package_fn_exprss = ArenaMap::new();
    let mut package_tenvs: ArenaMap<PackageId, _> = ArenaMap::new();
    let mut source_cache = SourceCache::new(interner);

    for pkg in &cfg.packages {
        if let Err(io_errors) = build_package_defs(
            &project_root.join(&pkg.name),
            &lowering_config,
            &mut packages,
            &mut package_bodies,
            &mut package_tenvs,
            &mut source_cache,
        ) {
            io_errors.into_iter().for_each(|io_err| io_err.report())
        };
    }

    for (package_id, _) in packages.iter() {
        let diagnostics = lower_package_bodies(
            package_id,
            &packages,
            &mut package_bodies[package_id],
            &mut package_tenvs[package_id],
            &interner,
            &lowering_config,
        );
        source_cache.report_diagnostics(diagnostics.iter());
    }

    ExitStatus::Success
}

pub fn build_package_defs(
    root_dir: &Path,
    config: &flux_hir::cfg::Config,
    packages: &mut Arena<flux_hir::PackageDefs>,
    package_bodies: &mut ArenaMap<PackageId, PackageBodies>,
    package_tenvs: &mut ArenaMap<PackageId, TEnv>,
    source_cache: &mut SourceCache,
) -> Result<PackageId, Vec<IOError>> {
    let cfg = match get_config(root_dir) {
        Err(err) => {
            return Err(vec![err]);
        }
        Ok(cfg) => cfg,
    };

    assert_eq!(cfg.packages.len(), 1);
    let pkg = &cfg.packages[0];

    let (entry_path, content) = match get_package_entry_file_path(root_dir, &pkg.name) {
        Ok(x) => x,
        Err(err) => {
            return Err(vec![err]);
        }
    };

    let interner = INTERNER
        .get()
        .unwrap_or_else(|| ice("interner should be initialized by now"));
    let entry_file_id = source_cache.add_input_file(&entry_path, content.clone());

    tracing::info!(package =? pkg, "building package definitions");
    let (package_defs, bodies, tenvs, diagnostics) =
        lower_package_defs(entry_file_id, &content, interner, source_cache, config);

    let package_id = packages.alloc(package_defs);
    source_cache.report_diagnostics(diagnostics.iter());
    package_tenvs.insert(package_id, tenvs);
    package_bodies.insert(package_id, bodies);

    Ok(package_id)
}

fn get_config(project_root: &Path) -> Result<Config, IOError> {
    let cfg_path = project_root.join(CFG_FILE_NAME);
    let content = fs::read_to_string(&cfg_path).map_err(|_error| {
        DriverError::ReadConfigFile {
            candidate: cfg_path.to_str().unwrap().to_string(),
        }
        .to_io_error()
    })?;
    Ok(flux_cfg::parse_cfg(&content))
}

fn get_package_entry_file_path(
    package_root: &Path,
    package_name: &str,
) -> Result<(String, String), IOError> {
    let file_path = package_root.join("src/main.flx");
    std::fs::read_to_string(&file_path)
        .map_err(|_error| {
            DriverError::ReadEntryFile {
                package: package_name.to_string(),
                candidate: file_path.to_str().unwrap().to_string(),
            }
            .to_io_error()
        })
        .and_then(|content| Ok((file_path.to_str().unwrap().to_string(), content)))
}
