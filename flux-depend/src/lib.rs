use std::{fs, path::Path, process::exit};

use flux_error::{FluxError, FluxErrorCode, FluxErrorReporting};
use flux_hir::HirModule;
use flux_parser::parse;
use flux_syntax::{
	ast,
	ast::{AstNode, Spanned},
};
use petgraph::{
	dot::Dot,
	graph::{DiGraph, NodeIndex},
	visit::EdgeRef,
};
use smol_str::SmolStr;

#[derive(Debug)]
pub enum DependencyType {
	SubModule,
	UseCrate,
	UseFunction(SmolStr),
	UseType(SmolStr),
}

pub type DependencyGraph = DiGraph<HirModule, DependencyType>;

fn find_node_child(
	graph: &DiGraph<HirModule, DependencyType>,
	parent: NodeIndex,
	name: &str,
) -> Option<NodeIndex> {
	let edges = graph.edges(parent);
	for edge in edges {
		let child = edge.target();
		if graph[child].name == name {
			return Some(child);
		}
	}
	None
}

fn find_function<'a>(
	module: &'a HirModule,
	name: &Spanned<SmolStr>,
) -> Option<&'a flux_hir::FnDecl> {
	let result = module.functions.iter().find_map(|f| {
		if let Some(n) = &f.name {
			if n.node == name.node {
				Some(f)
			} else {
				None
			}
		} else {
			None
		}
	});

	if let Some(f) = result {
		return Some(f);
	}
	None
}

fn find_type<'a>(module: &'a HirModule, name: &Spanned<SmolStr>) -> Option<&'a flux_hir::TypeDecl> {
	let result = module.types.iter().find_map(|ty| {
		if ty.name.node == name.node {
			Some(ty)
		} else {
			None
		}
	});

	if let Some(ty) = result {
		return Some(ty);
	}
	None
}

fn get_node_idx_from_use_path(
	graph: &DiGraph<HirModule, DependencyType>,
	path: &Vec<Spanned<SmolStr>>,
) -> Result<(NodeIndex, DependencyType), FluxError> {
	if path[0].node != "pkg" {
		panic!("`use` paths that don't begin with `pkg` are not yet supported");
	}

	let path_string = path
		.clone()
		.iter()
		.map(|x| x.node.clone())
		.collect::<Vec<SmolStr>>()
		.join("::");
	let mut path: Vec<Spanned<SmolStr>> = path.iter().rev().map(|x| x.clone()).collect();
	path.pop();
	let mut idx = NodeIndex::new(0);
	loop {
		if let Some(name) = path.last() {
			if let Some(m) = find_node_child(graph, idx, name) {
				idx = m;
				path.pop();
			} else {
				break;
			}
		} else {
			break;
		}
	}

	if path.len() > 0 {
		let name = path.last().unwrap();
		if path.len() > 1 {
			return Err(
				FluxError::default()
					.with_code(FluxErrorCode::UnresolvedUse)
					.with_msg(format!("could not find module `{}` in `use`", name.node)),
			);
		}

		let ty_res = find_type(&graph[idx], name);
		let f_res = find_function(&graph[idx], name);
		let name_string = name.node.clone();
		match (ty_res, f_res) {
			(Some(ty), Some(f)) => {
				return Err(
					FluxError::default()
						.with_msg(format!(
							"could not `use` `{}` as it is ambiguous",
							name_string
						))
						.with_primary(
							format!("could not `use` `{}` as it is ambiguous", name_string),
							Some(name.span.clone()),
						)
						.with_label(
							format!("type `{}` defined here", name_string),
							Some(ty.public.span.clone()),
						)
						.with_label(
							format!("function `{}` defined here", name_string),
							Some(f.public.span.clone()),
						),
				)
			}
			(Some(ty), None) => {
				if ty.public.node {
					return Ok((idx, DependencyType::UseType(name_string)));
				} else {
					return Err(
						FluxError::default()
							.with_code(FluxErrorCode::UnresolvedUse)
							.with_msg(format!("type `{}` defined as private", name_string))
							.with_primary(
								format!("cannot `use` private type `{}`", name_string),
								Some(name.span.clone()),
							)
							.with_label(
								format!("type `{}` defined as private here", name_string),
								Some(ty.public.span.clone()),
							)
							.with_note(format!(
								"(hint) make this public by adding the `pub` keyword before `type`"
							)),
					);
				}
			}
			(None, Some(f)) => {
				if f.public.node {
					return Ok((idx, DependencyType::UseFunction(name_string)));
				} else {
					return Err(
						FluxError::default()
							.with_code(FluxErrorCode::UnresolvedUse)
							.with_msg(format!("function `{}` defined as private", name_string))
							.with_primary(
								format!("cannot `use` private function `{}`", name_string),
								Some(name.span.clone()),
							)
							.with_label(
								format!("function `{}` defined as private here", name_string),
								Some(f.public.span.clone()),
							)
							.with_note(format!(
								"(hint) make this public by adding the `pub` keyword before `fn`"
							)),
					);
				}
			}
			(None, None) => {
				return Err(
					FluxError::default()
						.with_code(FluxErrorCode::UnresolvedUse)
						.with_msg(format!(
							"could not find type or function `{}` in module `{}`",
							name_string, graph[idx].name
						))
						.with_primary(
							format!(
								"could not find type or function `{}` in module `{}`",
								name_string, graph[idx].name
							),
							Some(name.span.clone()),
						),
				);
			}
		}
	}

	Ok((idx, DependencyType::UseCrate))
}

fn add_use_edges(graph: &mut DependencyGraph, err_reporting: &mut FluxErrorReporting) {
	let mut new_edges = vec![];
	for start in graph.node_indices() {
		let hir_module = &graph[start];
		for u in &hir_module.uses {
			let idx = get_node_idx_from_use_path(&graph, &u.path);
			if let Some(err) = idx.as_ref().err() {
				err_reporting.report(&[err.clone()]);
				break;
			}
			new_edges.push((start, idx.unwrap()));
		}
	}
	for (a, (b, dependency_type)) in new_edges {
		graph.add_edge(a, b, dependency_type);
	}
}

pub fn create_dot_file(graph: &DependencyGraph, path: &str) -> Result<(), std::io::Error> {
	// let s = format!("{:?}", Dot::new(graph));
	fs::write(path, "")
}
