use flux_span::FileSpanned;

use super::*;

impl<'a, 'pkgs> LowerCtx<'a, 'pkgs> {
    pub(super) fn verify_struct_expr_fields(
        &mut self,
        struct_decl: &StructDecl,
        struct_expr: &StructExpr,
    ) {
        let mut missing_fields = vec![];
        for field_decl in struct_decl.fields.iter() {
            let field_expr = struct_expr
                .fields
                .iter()
                .find(|expr| expr.name.inner == field_decl.name.inner);
            match field_expr {
                Some(field_expr) => {
                    let span = self.tckh.tenv.get_filespan(&field_expr.val.tid);
                    self.tckh
                        .unify(field_decl.ty.inner, field_expr.val.tid, span)
                        .unwrap_or_else(|err| self.diagnostics.push(err));
                }
                None => missing_fields.push(field_decl.name.clone()),
            }
        }

        if !missing_fields.is_empty() {
            self.diagnostics.push(
                LowerError::MissingFieldsInStructExpr {
                    struct_name: struct_expr.path.to_string(self.interner),
                    struct_name_file_span: struct_expr.path.span.in_file(self.file_id),
                    missing_fields: missing_fields
                        .iter()
                        .map(|name| self.interner.resolve(&name).to_string())
                        .collect(),
                    missing_fields_file_span: struct_expr.fields.span.in_file(self.file_id),
                }
                .to_diagnostic(),
            );
        }
    }

    pub(super) fn verify_call_args(
        &mut self,
        args: FileSpanned<&[TypeId]>,
        signature: FileSpanned<&[TypeId]>,
    ) {
        let args_count = args.len();
        let sig_count = signature.len();
        if args_count != sig_count {
            self.diagnostics.push(
                LowerError::IncorrectNumberOfArgs {
                    got_num: args_count,
                    got_num_file_span: args.to_file_span(),
                    expected_num: sig_count,
                    expected_num_file_span: signature.to_file_span(),
                }
                .to_diagnostic(),
            );
        }
    }
}
