use std::collections::HashMap;

use flux_hir::{FnDecl, FnParam, HirModule};
use flux_syntax::ast::Spanned;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Documentation, Position, Range};

use crate::semantic_tokens::flux_range_to_position;

// TODO: line number is the incorrect approach
// we need to convert `position` to an offset and compare those
pub fn get_completion_items(
	hir_module: &HirModule,
	position: &Position,
	src: &str,
) -> Vec<CompletionItem> {
	let mut names = vec![];
	let mut closest_function: Option<(Range, &FnDecl)> = None;
	hir_module.functions.iter().for_each(|f| {
		if let Some(name) = &f.name {
			let name_pos = flux_range_to_position(name.span.range, src);
			if name_pos.start.line < position.line {
				if let Some(closest) = closest_function {
					if name_pos.start.line > closest.0.start.line {
						closest_function = Some((name_pos, f));
					}
				} else {
					closest_function = Some((name_pos, f));
				}
			}
			names.push(CompletionItem {
				label: name.to_string(),
				kind: Some(CompletionItemKind::FUNCTION),
				detail: function_to_detail(f),
				..Default::default()
			});
		}
	});

	eprintln!("{:?}", closest_function); // we need closest function to store the function declaration, not range

	if let Some(closest_function) = closest_function {
		for stmt in &closest_function.1.block {
			if let Some(stmt) = stmt {
				match &stmt.node {
					flux_hir::Stmt::VarDecl(var) => names.push(CompletionItem {
						label: var.name.to_string(),
						kind: Some(CompletionItemKind::VARIABLE),
						detail: Some(format!("{}", var.ty.node)),
						..Default::default()
					}),
					_ => (),
				}
			}
		}
	}

	// 1. collect all functions
	// 2. figure out what function you're in based on position
	// 3. collect locals
	//  	* filter based on if they've been defined yet

	names
}

fn function_to_detail(f: &flux_hir::FnDecl) -> Option<String> {
	if let Some(name) = &f.name {
		let s = format!(
			"{}({}) -> {}",
			name.as_str(),
			function_params_to_string(&f.params),
			f.return_type.node
		);
		return Some(s);
	}
	None
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
