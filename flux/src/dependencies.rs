use flux_error::{FluxError, FluxErrorReporting};
use flux_hir::HirModule;
use flux_syntax::ast::Spanned;
use petgraph::{
	graph::{DiGraph, NodeIndex},
	visit::EdgeRef,
};

#[derive(Debug)]
pub enum DependencyType {
	SubModule,
	UseCrate,
	UseFunction(String),
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

fn find_function(module: &HirModule, name: &Spanned<String>) -> Result<(), FluxError> {
	let result = module.functions.iter().find_map(|f| {
		if let Some(n) = &f.name {
			if *n == name.node {
				Some(f.clone())
			} else {
				None
			}
		} else {
			None
		}
	});

	if let Some(f) = result {
		if !f.public.node {
			return Err(
				FluxError::default()
					.with_msg(format!("cannot `use` private function `{}`", name.node))
					.with_primary(
						format!(
							"module `{}` defines `{}` as private",
							module.name, name.node
						),
						Some(name.span.clone()),
					),
			);
		}
	} else if result.is_none() {
		return Err(
			FluxError::default()
				.with_msg(format!(
					"could not find function `{}` in module `{}`",
					name.node, module.name
				))
				.with_primary(
					format!(
						"could not find function `{}` in module `{}`",
						name.node, module.name
					),
					Some(name.span.clone()),
				),
		);
	}

	Ok(())
}

fn get_node_idx_from_use_path(
	graph: &DiGraph<HirModule, DependencyType>,
	path: &Vec<Spanned<String>>,
) -> Result<(NodeIndex, DependencyType), FluxError> {
	if path[0].node != "pkg" {
		panic!("`use` paths that don't begin with `pkg` are not yet supported");
	}

	let mut path: Vec<Spanned<String>> = path.iter().rev().map(|x| x.clone()).collect();
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
				FluxError::default().with_msg(format!("could not find module `{}` in `use`", name.node)),
			);
		}

		find_function(&graph[idx], name)?;
		return Ok((idx, DependencyType::UseFunction(name.node.clone())));
	} else {
		Ok((idx, DependencyType::UseCrate))
	}
}

pub fn add_use_edges(graph: &mut DependencyGraph, err_reporting: &mut FluxErrorReporting) {
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
