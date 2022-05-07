// use mir::MirPackage;
use flux_ast::AST;
use flux_error::{filesystem::FileId, *};
use std::collections::HashMap;

mod mir;

pub struct MirContext<'a> {
	ast: &'a AST,
	functions: Vec<mir::FnDecl>,
	cur_function: usize,
}

impl<'a> MirContext<'a> {
	pub fn new(ast: &'a AST) -> Self {
		Self {
			ast,
			functions: vec![],
			cur_function: 0,
		}
	}

	pub fn lower_functions(&mut self) {
		for f in &self.ast.functions {
			self.lower_function(f);
		}
	}

	fn lower_function(&mut self, function: &flux_ast::FnDecl) {
		let mut f = mir::FnDecl::new(function.name.to_string(), function.params.clone());
		let mut entry = f.new_block();
		for stmt in &function.block {
			entry.lower_stmt(&stmt);
		}
		f.blocks.push(entry);
		println!("{:#?}", f);
	}
}

fn type_expr_to_mir_type(ty: &flux_ast::Expr) -> mir::Type {
	match ty {
		flux_ast::Expr::PrimitiveType(x) => match x.kind {
			flux_ast::PrimitiveKind::I64 => mir::Type::I64,
			flux_ast::PrimitiveKind::U64 => mir::Type::U64,
			flux_ast::PrimitiveKind::I32 => mir::Type::I32,
			flux_ast::PrimitiveKind::U32 => mir::Type::U32,
			flux_ast::PrimitiveKind::I16 => mir::Type::I16,
			flux_ast::PrimitiveKind::U16 => mir::Type::U16,
			flux_ast::PrimitiveKind::I8 => mir::Type::I8,
			flux_ast::PrimitiveKind::U8 => mir::Type::U8,
			_ => mir::Type::I32,
		},
		_ => mir::Type::I32,
	}
}

pub fn generate_mir(file_ast_map: &HashMap<FileId, AST>, err_reporting: &mut PIErrorReporting) {
	// Assume that fileID of entry file is `0`
	let entry_fileid: FileId = FileId(0);
	let main = file_ast_map.get(&entry_fileid).expect("could not get file");

	let mut ctx = MirContext::new(&main);
	ctx.lower_functions();

	// let _ = lower_functions(&main);

	// for (k, v) in file_ast_map {
	// 	// println!("{:?} {:?}", k, v);
	// 	println!("{:?}", v.name);
	// 	println!("{:?}", err_reporting.get_filename(*k));
	// }

	// MirPackage {}
}
