use flux_typesystem::{TChecker, TEnv};
use la_arena::{Idx, RawIdx};

use super::*;

pub struct ModuleBodyContext<'a> {
    tchk: TChecker,
    module: &'a mut Module,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
    mod_namespace: &'a HashMap<Spur, ModuleId>,
    function_namespace: &'a HashMap<Spur, (FunctionId, ModuleId)>,
}

impl<'a> ModuleBodyContext<'a> {
    pub fn new(
        module: &'a mut Module,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
        mod_namespace: &'a HashMap<Spur, ModuleId>,
        function_namespace: &'a HashMap<Spur, (FunctionId, ModuleId)>,
    ) -> Self {
        Self {
            tchk: TChecker::new(TEnv::new(string_interner)),
            module,
            string_interner,
            type_interner,
            mod_namespace,
            function_namespace,
        }
    }

    pub fn lower_expr(&mut self, expr: Option<ast::Expr>) -> ExprIdx {
        let expr =
            expr.expect("internal compiler error: missing node that should always be emitted");
        if expr.is_poisoned() {
            self.module
                .exprs
                .alloc(Expr::Poisoned.at(expr.range().to_span()))
        } else {
            match expr {
                ast::Expr::BlockExpr(block) => self.lower_block_expr(block),
                ast::Expr::FloatExpr(float) => self.lower_float_expr(float),
                ast::Expr::IntExpr(int) => self.lower_int_expr(int),
                ast::Expr::PathExpr(path) => self.lower_path_expr(path),
                ast::Expr::StructExpr(strukt) => self.lower_struct_expr(strukt),
                _ => todo!(
                    "internal compiler error: unhandled expression type: {:#?}",
                    expr
                ),
            }
        }
        // lower_node(
        //     expr,
        //     |expr| {
        //         self.module
        //             .exprs
        //             .alloc(Expr::Poisoned.at(expr.range().to_span()))
        //     },
        //     |expr| match expr {
        //         ast::Expr::BlockExpr(block) => self.lower_block_expr(block),
        //         ast::Expr::FloatExpr(float) => self.lower_float_expr(float),
        //         ast::Expr::IntExpr(int) => self.lower_int_expr(int),
        //         ast::Expr::PathExpr(path) => self.lower_path_expr(path),
        //         ast::Expr::StructExpr(strukt) => self.lower_struct_expr(strukt),
        //         _ => todo!(
        //             "internal compiler error: unhandled expression type: {:#?}",
        //             expr
        //         ),
        //     },
        // )
    }

    fn lower_block_expr(&mut self, block: ast::BlockExpr) -> ExprIdx {
        let span = block.range().to_span();
        let exprs: Vec<_> = block
            .stmts()
            .map(|stmt| match stmt {
                ast::Stmt::ExprStmt(expr) => self.lower_expr(expr.expr()),
                ast::Stmt::LetStmt(let_expr) => self.lower_let_expr(let_expr),
            })
            .collect();
        let block = self
            .module
            .exprs
            .alloc(Expr::Block(Block::new(exprs)).at(span));
        block
    }

    fn lower_float_expr(&mut self, float: ast::FloatExpr) -> ExprIdx {
        let span = float.range().to_span();
        let value_str = match float.v() {
            Some(v) => self
                .string_interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => return self.module.exprs.alloc(Expr::Poisoned.at(span)),
        };
        let value: Spanned<f64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        self.module.exprs.alloc(Expr::Float(value.inner).at(span))
    }

    fn lower_int_expr(&mut self, int: ast::IntExpr) -> ExprIdx {
        let span = int.range().to_span();
        let value_str = match int.v() {
            Some(v) => self
                .string_interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => return self.module.exprs.alloc(Expr::Poisoned.at(span)),
        };
        let value: Spanned<u64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        self.module.exprs.alloc(Expr::Int(value.inner).at(span))
    }

    fn lower_path_expr(&mut self, path: ast::PathExpr) -> ExprIdx {
        let span = path.range().to_span();
        let segments = path
            .segments()
            .map(|segment| segment.text_key().at(segment.text_range().to_span()));
        let path = Path::from_segments(segments);
        let expr = Expr::Path(path);
        self.module.exprs.alloc(expr.at(span))
    }

    fn lower_struct_expr(&mut self, strukt: ast::StructExpr) -> ExprIdx {
        let span = strukt.range().to_span();
        let path = lower_path(strukt.path());
        let field_list = lower_node(
            strukt.field_list(),
            |_| vec![],
            |strukt| {
                strukt
                    .fields()
                    .map(|field| self.lower_struct_expr_field(field))
                    .collect()
            },
        );
        let strukt = Struct::new(path.inner, field_list);
        self.module.exprs.alloc(Expr::Struct(strukt).at(span))
    }

    fn lower_struct_expr_field(&mut self, field: ast::StructExprField) -> (Name, ExprIdx) {
        let name = lower_name(field.name(), self.string_interner);
        let val = self.lower_expr(field.val());
        (name, val)
    }

    fn lower_let_expr(&mut self, let_expr: ast::LetStmt) -> ExprIdx {
        let span = let_expr.range().to_span();
        let name = lower_name(let_expr.name(), self.string_interner);
        let lhs_ty = let_expr.ty().map_or(
            self.type_interner.intern(Type::Unknown).at(name.span),
            |ty| {
                lower_type(
                    Some(ty),
                    &GenericParamList::empty(),
                    span,
                    self.string_interner,
                    self.type_interner,
                )
            },
        );
        let expr = self.lower_expr(let_expr.value());
        let let_expr = self
            .module
            .exprs
            .alloc(Expr::Let(Let::new(name, lhs_ty, expr.ty_unknown())).at(span));
        let_expr
    }

    pub fn lower_bodies(&mut self) {
        for i in 0..self.module.functions.len() {
            let f = &self.module.functions[Idx::from_raw(RawIdx::from(i as u32))];
            self.lower_expr(f.ast.body());
        }
        // self.module.functions.iter().for_each(|(_, f)| {
        //     self.lower_expr(f.ast.body());
        // });
    }
}
