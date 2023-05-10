// use flux_hir::{hir, DefMap, ItemTree, LoweredBodies};
// use lasso::ThreadedRodeo;
// use lower::{LoweringCtx, ToValue};
// use value::Op;

// mod graph;
// mod lower;
// mod region;
// mod r#type;
// mod value;

// pub fn lower_def_map(
//     def_map: &DefMap,
//     bodies: &LoweredBodies,
//     string_interner: &'static ThreadedRodeo,
// ) {
//     def_map.item_trees.iter().for_each(|(modue_id, item_tree)| {
//         let ctx: LoweringCtx = LoweringCtx::new(bodies, item_tree, modue_id, string_interner);
//         lower_item_tree(item_tree, &ctx);
//     });
// }

// fn lower_item_tree(item_tree: &ItemTree, ctx: &LoweringCtx) {
//     let vals: Vec<_> = item_tree
//         .top_level
//         .iter()
//         .map(|item| match item {
//             flux_hir::ModItem::Apply(_) => todo!(),
//             flux_hir::ModItem::Enum(_) => todo!(),
//             // flux_hir::ModItem::Function(f) => item_tree[*f].to_value(ctx),
//             flux_hir::ModItem::Function(f) => f.to_value(ctx),
//             flux_hir::ModItem::Mod(_) => todo!(),
//             flux_hir::ModItem::Struct(_) => todo!(),
//             flux_hir::ModItem::Trait(_) => todo!(),
//             flux_hir::ModItem::Use(_) => todo!(),
//         })
//         .collect();
// }

// fn hir_op_to_mir_op(hir_op: hir::Op) -> Op {
//     match hir_op {
//         hir::Op::Eq => Op::Eq,
//         hir::Op::Plus => Op::Sum,
//     }
// }
