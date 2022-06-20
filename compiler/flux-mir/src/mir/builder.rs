use std::{cell::RefCell, rc::Rc};

use super::{
	Block, BlockID, Br, BrNZ, FnDecl, ICmp, ICmpKind, Instruction, MirID, RValue, Ret, StackAlloc,
	Store, Type,
};

macro_rules! declare_instrs_wo_ids {
	(
    $(
      $method:ident :
      $name:ident {
        $(
          $field_name:ident : $field_ty:ty
        ),*
        $(,)?
      };
    )*
  ) => {
    $(
      pub fn $method(&mut self, $($field_name : $field_ty),*) {
        let mut block = self.cur_block.as_ref().unwrap().borrow_mut();
				block
					.instrs
					.push(Instruction::$name($name { $($field_name),* }));
      }
    )*
  };
}

macro_rules! declare_instrs_with_ids {
	(
    $(
      $method:ident :
      $name:ident {
        $(
          $field_name:ident : $field_ty:ty
        ),*
        $(,)?
      };
    )*
  ) => {
    $(
      pub fn $method(&mut self, $($field_name : $field_ty),*) -> MirID {
        let mut block = self.cur_block.as_ref().unwrap().borrow_mut();
        let id = MirID(block.id_count);
				block.id_count += 1;
				block
					.instrs
					.push(Instruction::$name($name { id, $($field_name),* }));
        id
      }
    )*
  };
}

macro_rules! declare_terminators {
	(
    $(
      $method:ident :
      $name:ident {
        $(
          $field_name:ident : $field_ty:ty
        ),*
        $(,)?
      };
    )*
  ) => {
    $(
      pub fn $method(&mut self, $($field_name : $field_ty),*) {
        let mut block = self.cur_block.as_ref().unwrap().borrow_mut();
				block.terminator = Some(Instruction::$name($name { $($field_name),* }));
      }
    )*
  };
}

pub struct Builder {
	pub cur_fn: Option<Rc<RefCell<FnDecl>>>,
	pub cur_block: Option<Rc<RefCell<Block>>>,
}

impl Default for Builder {
	fn default() -> Self {
		Self {
			cur_fn: None,
			cur_block: None,
		}
	}
}

impl Builder {
	pub fn append_new_block(&mut self) -> Rc<RefCell<Block>> {
		let f = self.cur_fn.as_ref().unwrap();
		f.borrow_mut().append_new_block()
	}

	pub fn append_existing_block(&mut self, block: Rc<RefCell<Block>>) {
		let f = self.cur_fn.as_ref().unwrap();
		f.borrow_mut().blocks.push(block);
	}

	pub fn new_block(&mut self) -> Rc<RefCell<Block>> {
		let f = self.cur_fn.as_ref().unwrap();
		f.borrow_mut().new_block()
	}

	declare_instrs_with_ids!(
		new_alloca: StackAlloc {
			ty: Type
		};
		new_icmp: ICmp {
			kind: ICmpKind,
			lhs: RValue,
			rhs: RValue,
		};
	);

	declare_instrs_wo_ids!(
		new_store: Store {
			ptr: MirID,
			val: RValue,
		};
	);

	declare_terminators!(
		new_br: Br {
			to: BlockID
		};
		new_brnz: BrNZ {
			val: RValue,
			then: BlockID,
			else_: BlockID,
		};
		new_ret: Ret {
			val: Option<RValue>
		};
	);
}
