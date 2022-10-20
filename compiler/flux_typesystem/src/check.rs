use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};
use lasso::ThreadedRodeo;

use crate::{diagnostics::TypeError, TEnv, TypeId, TypeKind};

#[derive(Debug)]
pub struct TChecker {
    pub tenv: TEnv,
}

impl TChecker {
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            tenv: TEnv::new(string_interner),
        }
    }

    pub fn unify(&mut self, a: TypeId, b: TypeId, span: InFile<Span>) -> Result<(), Diagnostic> {
        use TypeKind::*;
        let a_kind = self.tenv.get_typekind_with_id(a);
        let b_kind = self.tenv.get_typekind_with_id(b);
        match (&a_kind.inner.inner, &b_kind.inner.inner) {
            (Unknown, _) => {
                self.tenv.set_type(a, b_kind.inner.inner);
                Ok(())
            }
            (_, _) => Err(self.type_mismatch(a, b, span).to_diagnostic()),
        }
    }

    fn type_mismatch(&self, a: TypeId, b: TypeId, span: InFile<Span>) -> TypeError {
        let a_span = self.tenv.get_type_filespan(a);
        let b_span = self.tenv.get_type_filespan(b);
        TypeError::TypeMismatch {
            a: FileSpanned::new(
                Spanned::new(self.tenv.fmt_ty_id(a), a_span.inner),
                a_span.file_id,
            ),
            b: FileSpanned::new(
                Spanned::new(self.tenv.fmt_ty_id(b), b_span.inner),
                b_span.file_id,
            ),
            span,
        }
    }
}
