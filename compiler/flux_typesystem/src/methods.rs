use flux_util::Word;

use crate::{FnSignature, TEnv, TypeKind};

pub struct MethodResolver {
    methods: Vec<(TypeKind, Vec<(Word, FnSignature)>)>,
}

impl MethodResolver {
    pub fn new(methods: Vec<(TypeKind, Vec<(Word, FnSignature)>)>) -> Self {
        Self { methods }
    }

    pub fn resolve_method(
        &self,
        ty: &TypeKind,
        name: &Word,
        tenv: &TEnv,
    ) -> Result<&FnSignature, ()> {
        self.methods
            .iter()
            .find(|(tkind, _)| tenv.types_unify(tkind, ty))
            .map(|(_, methods)| {
                methods.iter().find_map(|(method_name, signature)| {
                    if method_name == name {
                        Some(signature)
                    } else {
                        None
                    }
                })
            })
            .flatten()
            .ok_or(())
    }
}
