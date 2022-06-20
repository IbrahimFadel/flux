use flux_hir::{
	hir::{FnDecl, FnParam},
	HirModule,
};
use flux_syntax::ast::Spanned;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Position};

use crate::position::position_to_offset;

pub fn get_completion_items(
	hir_module: &HirModule,
	position: &Position,
	src: &str,
) -> Vec<CompletionItem> {
	let mut names = vec![];
	// let mut closest_function: Option<(u32, &FnDecl)> = None;
	// hir_module.functions.iter().for_each(|f| {
	// 	if let Some(name) = &f.name {
	// 		let off = position_to_offset(position, src);
	// 		if u32::from(name.span.range.end()) < off {
	// 			if let Some(closest) = closest_function {
	// 				if off > closest.0 {
	// 					closest_function = Some((off, f));
	// 				}
	// 			} else {
	// 				closest_function = Some((off, f));
	// 			}
	// 		}
	// 		names.push(CompletionItem {
	// 			label: name.to_string(),
	// 			kind: Some(CompletionItemKind::FUNCTION),
	// 			detail: function_to_detail(f),
	// 			..Default::default()
	// 		});
	// 	}
	// });

	// if let Some(closest_function) = closest_function {
	// 	for stmt in &closest_function.1.block.0 {
	// 		if let Some(stmt) = stmt {
	// 			match &stmt.node {
	// 				flux_hir::Stmt::VarDecl(var) => names.push(CompletionItem {
	// 					label: var.name.to_string(),
	// 					kind: Some(CompletionItemKind::VARIABLE),
	// 					detail: Some(format!("{}", var.ty.node)),
	// 					..Default::default()
	// 				}),
	// 				_ => (),
	// 			}
	// 		}
	// 	}
	// }

	// 1. collect all functions
	// 2. figure out what function you're in based on position
	// 3. collect locals
	//  	* filter based on if they've been defined yet

	names
}

fn function_to_detail(f: &flux_hir::hir::FnDecl) -> String {
	format!(
		"{}({}) -> {}",
		f.name.as_str(),
		function_params_to_string(&f.params),
		f.return_type.node
	)
}

fn function_params_to_string(params: &[Spanned<FnParam>]) -> String {
	let mut s = vec![];
	for param in params {
		if let Some(name) = &param.name {
			s.push(format!("{} {}", param.ty.node, name));
		}
	}
	s.join(", ")
}
