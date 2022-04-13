use mir::MirPackage;
use pi_ast::AST;
use pi_error::{filesystem::FileId, *};
use std::collections::HashMap;

mod mir;

//TODO Need MIRBuilder class to keep track of tags
// builder.new_function();
// builder.new_block();
// builder.new_stack_alloc();

pub fn generate_mir(
	file_ast_map: &HashMap<FileId, AST>,
	err_reporting: &mut PIErrorReporting,
) -> MirPackage {
	// Assume that fileID of entry file is `0`
	let entry_fileid: FileId = FileId(0);
	let main = file_ast_map.get(&entry_fileid).expect("could not get file");

	let mut ctx = mir::MirContext::new(&main);
	ctx.lower_functions();

	// let _ = lower_functions(&main);

	// for (k, v) in file_ast_map {
	// 	// println!("{:?} {:?}", k, v);
	// 	println!("{:?}", v.name);
	// 	println!("{:?}", err_reporting.get_filename(*k));
	// }

	MirPackage {}
}

// pub fn generate_mir(ast: &AST) {
// 	let m = MIRContext::new();
// 	for f in &ast.functions {
// 		// lower_fn(f);
// 		let params: Vec<Type> = f
// 			.params
// 			.into_iter()
// 			.map(|p| type_expr_to_mir_type(&p.type_))
// 			.collect();
// 		m.new_function(type_expr_to_mir_type(&f.ret_ty), f.name.clone(), params);
// 	}
// }

// fn lower_fn(f: &FnDecl) {
// 	for stmt in &f.block {
// 		let x = lower_stmt(stmt);
// 		println!("{:?}", x);
// 	}
// }

// fn lower_stmt(stmt: &Stmt) -> Vec<Instruction> {
// 	match stmt {
// 		Stmt::VarDecl(x) => lower_var_decl_stmt(x),
// 		_ => vec![],
// 	}
// }

// fn lower_var_decl_stmt(var: &VarDecl) -> Vec<Instruction> {
// 	let mut instructions = vec![];

// 	let ty = type_expr_to_mir_type(&var.type_);
// 	for _ in &var.names {
// 		// instructions.push(mir::Instruction::StackAlloc(ty.clone()));
// 		// instructions.push(assign);
// 	}

// 	return instructions;
// }

// fn expr_to_mir_rvalue(v: &pi_ast::Expr) -> mir::RValue {
// 	match v {}
// }
