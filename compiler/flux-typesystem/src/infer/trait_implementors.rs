use std::collections::HashMap;

use smol_str::SmolStr;

use crate::r#type::TypeId;

#[derive(Debug)]
pub struct TraitImplementorTable {
	// trait name -> ( impltor name -> ([trait_type_params], [impltor_type_params]) )
	table: HashMap<SmolStr, HashMap<SmolStr, Vec<(Vec<TypeId>, Vec<TypeId>)>>>,
}

impl TraitImplementorTable {
	pub fn new() -> Self {
		Self {
			table: HashMap::new(),
		}
	}

	pub fn insert_implementor(
		&mut self,
		trait_name: SmolStr,
		trait_ty_params: &[TypeId],
		impltor_name: SmolStr,
		impltor_ty_params: &[TypeId],
	) {
		let trait_implementors = self.table.entry(trait_name).or_insert(HashMap::new());
		let type_params_vec = trait_implementors.entry(impltor_name).or_insert(vec![]);
		type_params_vec.push((trait_ty_params.to_vec(), impltor_ty_params.to_vec()));
	}

	pub fn get_trait_implentations_for_type(
		&self,
		trait_name: &(SmolStr, Vec<TypeId>),
		type_name: &SmolStr,
	) -> Option<&Vec<(Vec<TypeId>, Vec<TypeId>)>> {
		match self.table.get(&trait_name.0) {
			Some(types_implementing_trait) => match types_implementing_trait.get(type_name) {
				Some(v) => Some(v),
				None => None,
			},
			None => None,
		}
	}
}
