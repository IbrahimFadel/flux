use flux_hir::hir;

use crate::mir::{EdgeKind, InputPort, Lambda, Node, Omega, Region, Rvsdg, Type};

impl Rvsdg {
	pub fn omega(&mut self) -> Omega {
		let omega_id = self.next_node();
		let state = self.create_output_port(omega_id, EdgeKind::State);
		let omega = Omega {
			node: omega_id,
			state,
		};
		self.nodes.insert(omega_id, Node::Omega(omega.clone()));
		omega
	}

	pub fn lambda(&mut self, fn_decl: &hir::FnDecl) -> Lambda {
		let lambda_id = self.next_node();

		let input_ports: Vec<InputPort> = fn_decl
			.params
			.0
			.iter()
			.map(|param| self.create_input_port(lambda_id, EdgeKind::Value, self.to_mir_ty(&param.ty)))
			.collect();
		let output_port = self.create_output_port(lambda_id, EdgeKind::Value);

		let lambda = Lambda::new(lambda_id, input_ports, output_port, Region::omega());
		self.nodes.insert(lambda_id, Node::Lambda(lambda.clone()));
		lambda
	}

	fn to_mir_ty(&self, ty: &hir::Type) -> Type {
		match ty {
			hir::Type::SInt(n) | hir::Type::UInt(n) => Type::Int(*n),
			_ => todo!(),
		}
	}
}
