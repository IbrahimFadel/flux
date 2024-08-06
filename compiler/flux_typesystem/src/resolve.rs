use std::mem;

use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_id::id;
use flux_util::{FileId, WithSpan};
use tracing::trace;

use crate::{diagnostics::TypeError, r#type::Restriction, ConcreteKind, TEnv, Type, TypeKind};

impl TEnv {
    pub fn resolve(
        &mut self,
        tid: id::Ty,
        file_id: FileId,
    ) -> Result<ConcreteKind, Vec<Diagnostic>> {
        trace!("resolving `{}", Into::<u32>::into(tid));
        let ty = self.get(tid);

        let mut diagnostics = vec![];
        let num_restrictions = ty.restrictions.len();
        loop {
            let ty = self.get_mut(tid);
            let restrictions = mem::take(&mut ty.restrictions);
            let reduced_restrictions: Vec<_> = restrictions
                .into_iter()
                .filter(|restriction| match restriction {
                    Restriction::Equals(other) => {
                        let unification_span = self.get_span(tid).in_file(file_id);
                        self.unify(tid, *other, unification_span)
                            .map_err(|err| diagnostics.push(err))
                            .is_err()
                    }
                    Restriction::Field(_) => todo!(),
                    Restriction::Trait(_) => todo!(),
                })
                .collect();
            let ty = self.get_mut(tid);
            ty.restrictions = reduced_restrictions;
            if num_restrictions == ty.restrictions.len() || ty.restrictions.is_empty() {
                break;
            }
        }

        let ty = self.get(tid);
        if ty.restrictions.is_empty() {
            return ty.kind.to_concrete(self).map_err(|err| vec![err]);
        }

        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        Err(vec![TypeError::CouldNotInfer {
            ty: (),
            ty_file_span: self.get_span(tid).in_file(file_id),
        }
        .to_diagnostic()])
    }
}

impl TypeKind {
    fn to_concrete(&self, tenv: &TEnv) -> Result<ConcreteKind, Diagnostic> {
        use TypeKind::*;
        match &self {
            Concrete(concrete_kind) => Ok(concrete_kind.clone()),
            Generic(_) => todo!(),
            ThisPath(this_path) => tenv
                .get(tenv.resolve_this_path(this_path))
                .kind
                .to_concrete(tenv),
            Ref(tid) => tenv.get(*tid).kind.to_concrete(tenv),
            Int => todo!(),
            Float => todo!(),
            Never => todo!(),
            Unknown => todo!(),
        }
    }
}
