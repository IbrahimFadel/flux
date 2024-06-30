// mod traits;
// mod unify;

use crate::env::TEnv;

#[derive(Debug)]
pub struct TChecker<'tenv> {
    pub tenv: &'tenv mut TEnv,
    // pub trait_applications: TraitApplicationTable,
}

impl<'tenv> TChecker<'tenv> {
    // pub fn new(tenv: &'tenv mut TEnv) -> Self {
    //     Self { tenv }
    // }
}

impl<'tenv> TChecker<'tenv> {
    // pub fn unify(&mut self, a: TypeId, b: TypeId) -> Result<(), Diagnostic> {
    //     let a_kind = self.tenv.get(&a);
    //     let b_kind = self.tenv.get(&b);
    //     match (a_kind, b_kind) {
    //         (_, _) => Err(self.type_mismatch(a, b)),
    //     }
    // }

    // fn type_mismatch(&self, a: TypeId, b: TypeId) -> Diagnostic {
    // todo!()
    // let a_file_span = self.tenv.get_type_filespan(a);
    // let b_file_span = self.tenv.get_type_filespan(b);

    // TypeError::TypeMismatch {
    //     a: self.tenv.fmt_ty_id(a),
    //     a_file_span,
    //     b: self.tenv.fmt_ty_id(b),
    //     b_file_span,
    //     span: (),
    //     span_file_span: unification_span,
    // }
    // .to_diagnostic()
    // }
}
