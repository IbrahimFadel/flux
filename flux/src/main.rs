use dependencies::{DependencyGraph, DependencyType};
use flux_error::{FluxError, FluxErrorReporting};
use flux_hir::HirModule;
use flux_parser::*;
use flux_syntax::{
	ast,
	ast::{AstNode, Spanned},
};
use flux_typecheck::*;
use petgraph::{
	dot::Dot,
	graph::{DiGraph, NodeIndex},
	visit::EdgeRef,
};
use std::{fs, path::Path, process::exit};

use crate::dependencies::add_use_edges;
const PROJECT_DIR: &str = "examples";

mod dependencies;

/// Given a mod name `foo`, search for the corresponding source file:
/// `./foo.flx` || `./foo/foo.flx`
/// OK((path it found, file content))
/// Err(FileNotFound)
fn find_path_with_mod_name(
	parent_dir: &str,
	name: &Spanned<String>,
) -> Result<(String, String), FluxError> {
	let src = fs::read_to_string(&format!("{}/{}.flx", parent_dir, name.node));
	if let Some(_) = src.as_ref().err() {
		let src = fs::read_to_string(&format!("{}/{}/{}.flx", parent_dir, name.node, name.node));
		if let Some(_) = src.as_ref().err() {
			Err(
				FluxError::default()
					.with_msg(format!("could not find module `{}`", name.node))
					.with_primary(
						format!("could not find module `{}`", name.node),
						Some(name.span.clone()),
					)
					.with_label(
						format!(
							"no such file `{}/{}.flx` or `{}/{}/{}.flx`",
							parent_dir, name.node, parent_dir, name.node, name.node
						),
						Some(name.span.clone()),
					),
			)
		} else {
			Ok((
				format!("{}/{}/{}.flx", parent_dir, name.node, name.node),
				src.unwrap(),
			))
		}
	} else {
		Ok((format!("{}/{}.flx", parent_dir, name.node), src.unwrap()))
	}
}

fn parse_file_and_submodules<'a>(
	parent_dir: &str,
	name: &Spanned<String>,
	err_reporting: &mut FluxErrorReporting,
	dependency_graph: &mut DependencyGraph,
	parent: NodeIndex,
) {
	let src = find_path_with_mod_name(parent_dir, name);
	if let Some(err) = src.as_ref().err() {
		err_reporting.report(&[err.clone()]);
		return;
	}
	let (path, src) = src.unwrap();

	let file_id = err_reporting
		.add_file(format!("{}/{}.flx", parent_dir, name.node), src.clone())
		.expect("could not add file");
	let cst = parse(src.as_str(), file_id);
	err_reporting.report(&cst.errors);
	let root = ast::Root::cast(cst.syntax()).unwrap();
	let hir_module = flux_hir::lower(name.node.clone(), root, file_id);
	err_reporting.report(&hir_module.db.errors);
	if cst.errors.len() + hir_module.db.errors.len() > 0 {
		exit(1);
	}

	let child = dependency_graph.add_node(hir_module.clone());
	dependency_graph.add_edge(parent, child, DependencyType::SubModule);

	for m in &hir_module.mods {
		let parent_dir = Path::new(&path).parent().unwrap();
		parse_file_and_submodules(
			parent_dir.to_str().unwrap(),
			&m.name,
			err_reporting,
			dependency_graph,
			child,
		);
	}
}

fn main() {
	let mut err_reporting = FluxErrorReporting::default();
	let src = fs::read_to_string(&format!("{}/{}", PROJECT_DIR, "main.flx")).unwrap();
	let entry_file_id = err_reporting
		.add_file(format!("{}/main.flx", PROJECT_DIR), src.clone())
		.expect("could not add file");
	let entry_cst = parse(src.as_str(), entry_file_id);
	err_reporting.report(&entry_cst.errors);
	let root = ast::Root::cast(entry_cst.syntax()).unwrap();
	let hir_module = flux_hir::lower(String::from("main"), root, entry_file_id);
	err_reporting.report(&hir_module.db.errors);
	if entry_cst.errors.len() + hir_module.db.errors.len() > 0 {
		exit(1);
	}

	let mut dependency_graph = DiGraph::new();
	let root = dependency_graph.add_node(hir_module.clone());

	for m in &hir_module.mods {
		parse_file_and_submodules(
			PROJECT_DIR,
			&m.name,
			&mut err_reporting,
			&mut dependency_graph,
			root,
		);
	}

	add_use_edges(&mut dependency_graph, &mut err_reporting);

	// let dot = format!("{:?}", Dot::new(&dependency_graph));
	// let res = fs::write("dependencies.dot", dot);
	// if let Some(err) = res.err() {
	// 	panic!("{}", err.to_string());
	// }
}
