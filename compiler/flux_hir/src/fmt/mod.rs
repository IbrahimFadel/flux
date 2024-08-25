use std::collections::HashSet;

use flux_diagnostics::{Diagnostic, DiagnosticCode, SourceCache};
use flux_id::{id, Map};
use flux_typesystem::{TEnv, TypeKind, Typed};
use flux_util::{FileId, WithSpan};

use crate::def::expr::Expr;

macro_rules! format_tid {
    ($tid:expr, $tenv:expr, $file_id:expr) => {
        format!("'{}: {}{}", Into::<u32>::into($tid), $tenv.fmt_tid($tid), {
            let tkind = $tenv.resolve($tid).unwrap_or(TypeKind::Unknown);
            if tkind != $tenv.get_inner($tid).kind {
                format!(" OR {}", $tenv.fmt_typekind(&tkind))
            } else {
                format!("")
            }
        })
        .file_span($file_id, $tenv.get_span($tid))
    };
}

pub(super) fn format_function_with_types(
    body: id::Expr,
    exprs: &Map<id::Expr, Typed<Expr>>,
    tenv: &mut TEnv,
    source_cache: &SourceCache,
    file_id: FileId,
) {
    let mut labels = vec![];
    let mut tids_formatted = HashSet::new();
    for expr in exprs.values() {
        if !tids_formatted.contains(&expr.tid) {
            labels.push(format_tid!(expr.tid, tenv, file_id));
        }
        tids_formatted.insert(expr.tid);
    }

    let diagnostic = Diagnostic::error(
        tenv.get(exprs.get(body).tid)
            .span
            .in_file(file_id)
            .to_file_span(),
        DiagnosticCode::CouldNotInfer,
        format!(""),
        labels,
    );
    let mut buf = Vec::new();
    source_cache.write_diagnostics_to_buffer(&vec![diagnostic], &mut buf);
    let s = String::from_utf8(buf).unwrap();
    println!("{s}");
}
