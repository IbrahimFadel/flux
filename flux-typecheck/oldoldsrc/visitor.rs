// use flux_ast::{
// 	BinOp, CallExpr, Expr, FloatLit, FnDecl, IntLit, Return, Stmt, StructExpr, VarDecl, AST,
// };

// use crate::PIResult;

// pub trait Data {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult;
// }

// pub trait Visitor {
// 	// Top Level Decls
// 	fn visit_ast(&mut self, ast: &mut AST) -> PIResult;
// 	fn visit_fn_decl(&mut self, fn_decl: &mut FnDecl) -> PIResult;

// 	// Stmts
// 	fn visit_stmt(&mut self, stmt: &mut Stmt) -> PIResult;
// 	fn visit_var_decl_stmt(&mut self, var: &mut VarDecl) -> PIResult;
// 	fn visit_ret_stmt(&mut self, ret: &mut Return) -> PIResult;

// 	// Exprs
// 	fn visit_expr(&mut self, expr: &mut Expr) -> PIResult;
// 	fn visit_int_expr(&mut self, int: &mut IntLit) -> PIResult;
// 	fn visit_float_expr(&mut self, float: &mut FloatLit) -> PIResult;
// 	fn visit_binop_expr(&mut self, binop: &mut BinOp) -> PIResult;
// 	fn visit_call_expr(&mut self, call: &mut CallExpr) -> PIResult;
// 	fn visit_struct_expr(&mut self, struct_expr: &mut StructExpr) -> PIResult;
// }

// impl Data for AST {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_ast(self)
// 	}
// }

// impl Data for FnDecl {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_fn_decl(self)
// 	}
// }

// impl Data for Stmt {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_stmt(self)
// 	}
// }

// impl Data for VarDecl {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_var_decl_stmt(self)
// 	}
// }

// impl Data for Return {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_ret_stmt(self)
// 	}
// }

// impl Data for Expr {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_expr(self)
// 	}
// }

// impl Data for IntLit {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_int_expr(self)
// 	}
// }

// impl Data for FloatLit {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_float_expr(self)
// 	}
// }

// impl Data for BinOp {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_binop_expr(self)
// 	}
// }

// impl Data for CallExpr {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_call_expr(self)
// 	}
// }

// impl Data for StructExpr {
// 	fn accept<V: Visitor>(&mut self, visitor: &mut V) -> PIResult {
// 		visitor.visit_struct_expr(self)
// 	}
// }

// // use flux_ast::{Expr, FnDecl, Stmt, VarDecl};

// // pub trait Visitor<T> {
// // 	fn visit(&mut self, t: &T);
// // }

// // pub trait Visitable: Sized {
// // 	fn accept<T>(&self, t: &mut T)
// // 	where
// // 		T: Visitor<Self>,
// // 	{
// // 		t.visit(self);
// // 	}
// // }

// // impl Visitable for FnDecl {}
// // impl Visitable for Stmt {}

// // struct Vis;

// // impl<T> Visitor<T> for Vis
// // where
// // 	T: Visitable,
// // {
// // 	fn visit(&mut self, _: &T) {
// // 		unimplemented!()
// // 	}
// // }

// // impl Visitor<FnDecl> for Vis {
// // 	fn visit(&mut self, t: &FnDecl) {
// // 		println!("visiting fn");
// // 	}
// // }

// // impl Visitor<Stmt> for Vis {
// // 	fn visit(&mut self, t: &Stmt) {
// // 		println!("visiting stmt");
// // 	}
// // }
