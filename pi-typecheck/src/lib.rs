use std::{borrow::BorrowMut, collections::HashMap, ops::Range};

use pi_ast::{
	BinOp, Expr, FloatLit, FnDecl, IntLit, PrimitiveKind, PrimitiveType, Return, Stmt, VarDecl, AST,
};
use pi_error::{filesystem::FileId, *};

struct TypecheckCtx<'a> {
	expecting_ty: Expr,
	errors: &'a mut Vec<PIError>,
	file_id: FileId,
}

impl<'a> TypecheckCtx<'a> {
	fn error(&self, msg: String, code: PIErrorCode, labels: Vec<(String, Range<usize>)>) -> PIError {
		PIError::new(msg, code, labels, self.file_id)
	}

	pub fn check(&mut self, ast: &mut AST) {
		for f in &mut ast.functions {
			self.check_fn(f);
		}
	}

	fn check_fn(&mut self, f: &mut FnDecl) {
		for stmt in &mut f.block {
			self.expecting_ty = f.ret_ty.clone();
			self.check_stmt(stmt);
		}
	}

	fn check_stmt(&mut self, stmt: &mut Stmt) {
		match stmt {
			Stmt::VarDecl(var) => self.check_var(var),
			Stmt::Return(ret) => self.check_ret(ret),
			_ => (),
		}
	}

	fn check_ret(&mut self, ret: &mut Return) {
		if let Some(x) = &mut ret.val {
			self.check_expr(x);
		}
	}

	fn check_var(&mut self, var: &mut VarDecl) {
		self.expecting_ty = var.type_.clone();

		if let Some(vals) = &mut var.values {
			for val in vals {
				self.check_expr(val);
			}
		}
	}

	fn check_expr(&mut self, expr: &mut Expr) {
		match expr {
			Expr::IntLit(int) => self.check_int_lit(int),
			Expr::FloatLit(float) => self.check_float_lit(float.borrow_mut()),
			Expr::BinOp(binop) => self.check_binop(binop),
			_ => (),
		}
	}

	fn check_binop(&mut self, binop: &mut BinOp) {
		self.check_expr(&mut *binop.x);
		self.check_expr(&mut *binop.y);
	}

	fn check_float_lit(&mut self, float: &mut FloatLit) {
		if let Some(prim) = TypecheckCtx::type_is_primitive(&self.expecting_ty) {
			let expected_bits = TypecheckCtx::primitive_kind_to_bits(&prim.kind);
			if float.bits != expected_bits {
				float.bits = expected_bits;
			}
		}
	}

	fn check_int_lit(&mut self, int: &mut IntLit) {
		if let Some(prim) = TypecheckCtx::type_is_primitive(&self.expecting_ty) {
			let expected_bits = TypecheckCtx::primitive_kind_to_bits(&prim.kind);
			let expected_signed = TypecheckCtx::primitive_kind_to_signedness(&prim.kind);
			if int.bits != expected_bits {
				int.bits = expected_bits;
			}
			if expected_signed == false && int.signed == true {
				let mut labels = vec![("expected unsigned integer".to_owned(), int.val_span.clone())];
				if expected_signed == false {
					labels.push((format!("unexpected `-`"), int.sign_span.clone()))
				}
				self.errors.push(self.error(
					format!("expected unsigned integer but got signed integer",),
					PIErrorCode::TypecheckUnexpectedSignednessInIntLit,
					labels,
				));
			}
		}
	}

	fn type_is_primitive(ty: &Expr) -> Option<&PrimitiveType> {
		match ty {
			Expr::PrimitiveType(prim) => Some(prim),
			_ => None,
		}
	}

	fn primitive_kind_to_bits(prim: &PrimitiveKind) -> u8 {
		match prim {
			PrimitiveKind::U64 | PrimitiveKind::I64 | PrimitiveKind::F64 => 64,
			PrimitiveKind::U32 | PrimitiveKind::I32 | PrimitiveKind::F32 => 32,
			PrimitiveKind::U16 | PrimitiveKind::I16 => 16,
			PrimitiveKind::U8 | PrimitiveKind::I8 => 8,
			_ => 32,
		}
	}

	fn primitive_kind_to_signedness(prim: &PrimitiveKind) -> bool {
		match prim {
			PrimitiveKind::U64 | PrimitiveKind::U32 | PrimitiveKind::U16 | PrimitiveKind::U8 => false,
			PrimitiveKind::I64
			| PrimitiveKind::I32
			| PrimitiveKind::I16
			| PrimitiveKind::I8
			| PrimitiveKind::F64
			| PrimitiveKind::F32 => true,
			_ => true,
		}
	}
}

pub fn typecheck_ast(file_ast_map: &mut HashMap<FileId, AST>) -> Vec<PIError> {
	let entry_fileid: FileId = FileId(0);
	let ast = file_ast_map
		.get_mut(&entry_fileid)
		.expect("could not get file");
	let mut errors = vec![];
	let mut ctx = TypecheckCtx {
		expecting_ty: Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::I32)),
		errors: &mut errors,
		file_id: entry_fileid,
	};

	ctx.check(ast);

	return errors;
}
