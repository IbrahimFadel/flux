use std::collections::HashMap;

use flux_diagnostics::ice;
use flux_id::id::{self, P};
use flux_util::{Path, Word};

use crate::r#type::Type;

pub struct TraitResolution {
    this_types: HashMap<P<id::ApplyDecl>, Type>,
    trait_applications: HashMap<P<id::TraitDecl>, Vec<(P<id::ApplyDecl>, TraitApplicationInfo)>>,
}

impl TraitResolution {
    pub fn new(
        this_types: HashMap<P<id::ApplyDecl>, Type>,
        trait_applications: HashMap<
            P<id::TraitDecl>,
            Vec<(P<id::ApplyDecl>, TraitApplicationInfo)>,
        >,
    ) -> Self {
        Self {
            // traits,
            this_types,
            trait_applications,
        }
    }

    pub fn get_this_type(&self, apply_id: &P<id::ApplyDecl>) -> &Type {
        self.this_types
            .get(apply_id)
            .unwrap_or_else(|| ice(format!("invalid `ApplyId`: {:?}", apply_id)))
    }

    pub fn get_trait_application(
        &self,
        trait_id: &P<id::TraitDecl>,
        apply_id: &P<id::ApplyDecl>,
    ) -> &TraitApplicationInfo {
        self.trait_applications
            .get(trait_id)
            .unwrap_or_else(|| ice(format!("invalid `TraitId`: {:?}", trait_id)))
            .iter()
            .find_map(|(application, app)| {
                if application == apply_id {
                    Some(app)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| ice(format!("invalid `ApplyId`: {:?}", apply_id)))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ThisCtx {
    TraitDecl(P<id::TraitDecl>),
    TraitApplication((P<id::TraitDecl>, P<id::ApplyDecl>)),
    TypeApplication(P<id::ApplyDecl>),
    None,
}

pub struct TraitApplicationInfo {
    assoc_types: Vec<(Word, Type)>,
}

impl TraitApplicationInfo {
    pub fn new(assoc_types: Vec<(Word, Type)>) -> Self {
        Self { assoc_types }
    }

    pub(crate) fn get_associated_type(&self, this_path: &Path<Word>) -> &Type {
        if this_path.len() != 1 {
            ice("`ThisPath` of associated type should have length of 1");
        }
        self.assoc_types
            .iter()
            .find_map(|(name, ty)| {
                if name == this_path.get_nth(0) {
                    Some(ty)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| ice(format!("no associated type `{:?}`", this_path.get_nth(0))))
    }
}
