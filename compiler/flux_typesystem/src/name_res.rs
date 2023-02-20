use flux_span::Spanned;
use lasso::Spur;
use std::collections::HashMap;

#[derive(Debug)]
pub struct NameResolver<'a> {
    // fn id, mod id
    function_namespace: &'a HashMap<Spur, (u32, u32)>,
    // struct id, mod id
    struct_namespace: &'a HashMap<Spur, (u32, u32)>,
}

impl<'a> NameResolver<'a> {
    pub fn new(
        function_namespace: &'a HashMap<Spur, (u32, u32)>,
        struct_namespace: &'a HashMap<Spur, (u32, u32)>,
    ) -> Self {
        Self {
            function_namespace,
            struct_namespace,
        }
    }

    pub(super) fn resolve_function_path(&self, path: &Spanned<Spur>) -> Option<(u32, u32)> {
        self.function_namespace.get(path).copied()
    }
}
