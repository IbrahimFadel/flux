pub(crate) static PRELUDE_SRC: &'static str = include_str!("prelude/prelude.flx");

// static BIN_OP_TRAIT_NAME_MAP: OnceLock<HashMap<Op, &'static str>> = OnceLock::new();
// static BIN_OP_TRAIT_MAP: OnceLock<HashMap<Op, ItemId>> = OnceLock::new();

// fn bin_op_trait_name_map() -> &'static HashMap<Op, &'static str> {
//     BIN_OP_TRAIT_NAME_MAP
//         .get_or_init(|| HashMap::from([(Op::Add, "Add"), (Op::Mul, "Mul"), (Op::CmpEq, "CmpEq")]))
// }

// pub(crate) fn bin_op_trait_map(
//     prelude_data: &ModuleData,
//     interner: &'static Interner,
// ) -> &'static HashMap<Op, ItemId> {
//     BIN_OP_TRAIT_MAP.get_or_init(|| {
//         bin_op_trait_name_map()
//             .iter()
//             .map(|(op, name)| {
//                 let (_, item_id) = &prelude_data.scope.items[&interner.get_or_intern_static(name)];
//                 (*op, item_id.clone())
//             })
//             .collect()
//     })
// }
