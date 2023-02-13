use flux_diagnostics::Diagnostic;
use flux_span::{InFile, Span};
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};

use crate::{diagnostics::TypeError, TypeId};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct TraitImplementation {
    trait_params: Vec<TypeId>,
    impltr_params: Vec<TypeId>,
}

impl TraitImplementation {
    pub fn new(trait_params: Vec<TypeId>, impltr_params: Vec<TypeId>) -> Self {
        Self {
            trait_params,
            impltr_params,
        }
    }
}

#[derive(Debug)]
pub(crate) struct TraitImplementationTable {
    /// Trait Name -> (Implementor Name -> Implementation)
    pub table: HashMap<Spur, HashMap<Spur, Vec<TraitImplementation>>>,
    /// Spans of types
    spans: HashMap<Spur, HashMap<Spur, HashMap<TraitImplementation, InFile<Span>>>>,
}

impl TraitImplementationTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            spans: HashMap::new(),
        }
    }

    // pub fn add_new_implementation(
    //     &mut self,
    //     trt: Spur,
    //     trait_params: Vec<TypeId>,
    //     impltr: Spur,
    //     impltr_params: Vec<TypeId>,
    //     span: InFile<Span>
    // ) -> Result<(), Diagnostic> {
    //     let entry = self
    //         .table
    //         .entry(trt)
    //         .or_insert_with(|| HashMap::new())
    //         .entry(impltr)
    //         .or_insert_with(|| vec![]);

    //     let new_trait_impl = TraitImplementation::new(trait_params, impltr_params);
    //     entry
    //         .iter()
    //         .find(|trait_impl| **trait_impl == new_trait_impl)
    //         .cloned()
    //         .and_then(|conflicting_trait_impl| {
    //             //
    //             let span = self.get_type_span(&trt, &impltr, &conflicting_trait_impl);
    //             let diagnostic = TypeError::ConflictingTraitImplementations { implementation_a_file_id: span.file_id, implementation_b_file_id: span.file_id, impl_a_trt: format!("{}{}", self.string_interner.resolve(&trt), if trait_params.is_empty() {
    //                 format!("")
    //             } else {
    //                 format!("<{}>", trait_params.iter().map(|id| ))
    //             }), impl_a_ty: (), impl_b_trt: (), impl_b_ty: () }
    //             Some(())
    //         });

    //     Ok(())
    // }

    pub(crate) fn get_type_span(
        &self,
        trt: &Spur,
        impltr: &Spur,
        implementation: &TraitImplementation,
    ) -> InFile<Span> {
        self.spans
            .get(trt)
            .expect("internal compiler error: span isn't stored for trait implementation")
            .get(impltr)
            .expect("internal compiler error: span isn't stored for trait implementation")
            .get(implementation)
            .cloned()
            .expect("internal compiler error: span isn't stored for trait implementation")
    }
}

#[derive(Debug)]
pub(crate) struct TraitSolver {
    pub implementation_table: TraitImplementationTable,
    cache: HashMap<Spur, Spur>,
}

impl TraitSolver {
    pub fn new() -> Self {
        Self {
            implementation_table: TraitImplementationTable::new(),
            cache: HashMap::new(),
        }
    }
}
