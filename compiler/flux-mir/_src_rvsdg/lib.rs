use flux_hir::HirModule;
use mir::Rvsdg;

mod build;
mod mir;

pub fn lower_hir_module(hir_module: &HirModule) {
	let mut rvsdg = Rvsdg::new();
	rvsdg.omega();

	hir_module.functions.iter().for_each(|fn_decl| {
		rvsdg.lambda(fn_decl);
	});
}
