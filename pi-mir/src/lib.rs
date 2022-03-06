// use mir::{Assign, Instruction, Type};
// use pi_ast::{FnDecl, Stmt, VarDecl};

// mod mir;

// pub fn generate_mir(fns: Vec<FnDecl>) {
// 	for f in &fns {
// 		lower_fn(f);
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

// fn type_expr_to_mir_type(ty: &pi_ast::Expr) -> Type {
// 	match ty {
// 		pi_ast::Expr::PrimitiveType(x) => match x.kind {
// 			pi_ast::PrimitiveKind::I64 => Type::I64,
// 			pi_ast::PrimitiveKind::I32 => Type::I32,
// 			_ => Type::I32,
// 		},
// 		_ => Type::I32,
// 	}
// }
