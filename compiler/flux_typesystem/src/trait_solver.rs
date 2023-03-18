use flux_span::{InFile, Span};
use lasso::Spur;
use std::collections::HashMap;

use crate::{TEnv, TypeId};

#[derive(Debug, Hash, Clone)]
pub(crate) struct TraitImplementation {
    /// First `TypeId` is the implementor, followed by the trait parameters, then the implementor parameters
    ids: Vec<TypeId>,
    num_trait_params: usize,
}

impl TEnv {
    // pub(crate) fn fmt_trait_implementation(
    //     &self,
    //     trait_name: &Spur,
    //     trait_implementation: &TraitImplementation,
    // ) -> String {
    //     format!(
    //         "apply {}{} to {}{}",
    //         self.string_interner.resolve(trait_name),
    //         if trait_implementation.get_trait_params().is_empty() {
    //             "".to_string()
    //         } else {
    //             format!(
    //                 "<{}>",
    //                 trait_implementation
    //                     .get_trait_params()
    //                     .iter()
    //                     .map(|param| self.fmt_ty_id(*param))
    //                     .join(", ")
    //             )
    //         },
    //         self.fmt_ty_id(trait_implementation.get_impltor()),
    //         if trait_implementation.get_impltor_params().is_empty() {
    //             "".to_string()
    //         } else {
    //             format!(
    //                 "<{}>",
    //                 trait_implementation
    //                     .get_impltor_params()
    //                     .iter()
    //                     .map(|param| self.fmt_ty_id(*param))
    //                     .join(", ")
    //             )
    //         }
    //     )
    // }
}

impl TraitImplementation {
    pub(crate) fn new(
        trait_params: Vec<TypeId>,
        impltor: TypeId,
        impltor_params: Vec<TypeId>,
    ) -> Self {
        let num_trait_params = trait_params.len();
        Self {
            ids: std::iter::once(impltor)
                .chain(trait_params)
                .chain(impltor_params)
                .collect(),
            num_trait_params,
        }
    }

    pub fn get_impltor(&self) -> TypeId {
        unsafe { *self.ids.get_unchecked(0) }
    }

    pub fn get_trait_params(&self) -> Vec<TypeId> {
        if self.num_trait_params == 0 {
            vec![]
        } else {
            unsafe {
                self.ids
                    .get_unchecked(1..self.num_trait_params + 1)
                    .to_vec()
            }
        }
    }

    pub fn get_impltor_params(&self) -> Vec<TypeId> {
        self.ids
            .get(self.num_trait_params..)
            .map(Vec::from)
            .unwrap_or_else(Vec::new)
    }
}

#[derive(Debug)]
pub(crate) struct TraitImplementationTable {
    /// A Map from the trait name to its implementors
    pub table: HashMap<Spur, Vec<TraitImplementation>>,
    // Spans of types
    pub spans: HashMap<Spur, Vec<(TraitImplementation, InFile<Span>)>>,
}

impl TraitImplementationTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            spans: HashMap::new(),
        }
    }

    pub(crate) fn set_type_span(
        &mut self,
        trt: Spur,
        implementation: TraitImplementation,
        span: InFile<Span>,
    ) {
        self.spans
            .entry(trt)
            .or_insert_with(Vec::new)
            .push((implementation, span));
    }
}

// #[derive(Debug)]
// pub(crate) struct TraitSolver {
//     pub implementation_table: TraitImplementationTable,
// }

// impl TraitSolver {
//     pub fn new() -> Self {
//         Self {
//             implementation_table: TraitImplementationTable::new(),
//         }
//     }

//     pub fn try_add_implementation(
//         &mut self,
//         trait_name: Spur,
//         implementation: TraitImplementation,
//     ) -> Result<(), Diagnostic> {
//         let implementations = self
//             .implementation_table
//             .table
//             .entry(trait_name)
//             .or_insert(vec![]);

//         for implemtation in implementations {}

//         Ok(())
//     }

//     fn are_trait_impls_equal(a: &TraitImplementation, b: &TraitImplementation) -> bool {
//         let impltor_a = a.get_impltor();
//         let impltor_b = b.get_impltor();
//     }
// }
