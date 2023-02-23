use flux_span::{InFile, Span};
use lasso::Spur;
use std::collections::HashMap;

use crate::TypeId;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct TraitImplementation {
    /// First `TypeId` is the implementor, followed by the trait parameters, then the implementor parameters
    ids: Vec<TypeId>,
    num_trait_params: usize,
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
                .chain(trait_params.into_iter())
                .chain(impltor_params)
                .collect(),
            num_trait_params,
        }
    }

    pub fn get_impltor(&self) -> TypeId {
        unsafe { *self.ids.get_unchecked(0) }
    }

    pub fn get_trait_params(&self) -> &[TypeId] {
        unsafe { self.ids.get_unchecked(1..self.num_trait_params) }
    }

    pub fn get_impltor_params(&self) -> &[TypeId] {
        unsafe { self.ids.get_unchecked(self.num_trait_params..) }
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

#[derive(Debug)]
pub(crate) struct TraitSolver {
    pub implementation_table: TraitImplementationTable,
    // cache: HashMap<Spur, Spur>,
}

impl TraitSolver {
    pub fn new() -> Self {
        Self {
            implementation_table: TraitImplementationTable::new(),
            // cache: HashMap::new(),
        }
    }
}
