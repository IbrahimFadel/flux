use std::fmt;

use super::{Block, BlockID, FnDecl, FnParam, ICmpKind, Instruction, MirID, RValue, Type};

impl fmt::Display for FnDecl {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"fn @{}({}) -> {} {{\n{}\n}}",
			self.name,
			self
				.params
				.iter()
				.map(|p| format!("{}", p))
				.collect::<Vec<_>>()
				.join(", "),
			self.ret_ty,
			self
				.blocks
				.iter()
				.map(|b| format!("{}", b.borrow()))
				.collect::<Vec<_>>()
				.join("\n")
		)
	}
}

impl fmt::Display for FnParam {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} {}", self.ty, self.name)
	}
}

impl fmt::Display for Block {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}:\n{}\n{}",
			self.id,
			self
				.instrs
				.iter()
				.map(|instr| format!("\t{}", instr))
				.collect::<Vec<_>>()
				.join("\n"),
			if let Some(terminator) = &self.terminator {
				format!("\t{}", terminator)
			} else {
				format!("\t<missing terminator>")
			}
		)
	}
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			Self::StackAlloc(alloc) => write!(f, "{} = alloc.stack {}", alloc.id, alloc.ty),
			Self::Store(store) => write!(f, "store {}, {}", store.val, store.ptr),
			Self::ICmp(icmp) => write!(
				f,
				"{} = icmp.{} {}, {}",
				icmp.id, icmp.kind, icmp.lhs, icmp.rhs
			),
			Self::BrNZ(brnz) => write!(f, "brnz {}, {}, {}", brnz.val, brnz.then, brnz.else_),
			Self::Br(br) => write!(f, "br {}", br.to),
			Self::Ret(ret) => write!(
				f,
				"ret {}",
				if let Some(v) = &ret.val {
					format!("{}", v)
				} else {
					format!("void")
				}
			),
			_ => write!(f, "unprintable instruction {:?}", self),
		}
	}
}

impl fmt::Display for ICmpKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Eq => write!(f, "eq"),
			Self::ULt => write!(f, "ult"),
			Self::SLt => write!(f, "slt"),
		}
	}
}

impl fmt::Display for RValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			Self::Int(int) => write!(f, "const.i{} {}", int.size, int.n),
			Self::F32(n) => write!(f, "const.f32 {}", n),
			Self::Local(id) => write!(f, "{}", id),
			_ => write!(f, "unprintable rval"),
		}
	}
}

impl fmt::Display for MirID {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "%v{}", self.0)
	}
}

impl fmt::Display for BlockID {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "%block{}", self.0)
	}
}

impl fmt::Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Type::Ptr(ptr) => write!(f, "{}*", *ptr),
			Type::Ident(ident) => write!(f, "{ident}"),
			Type::StructTy(struct_ty) => {
				write!(f, "\\{{ ")?;
				for (i, ty) in struct_ty.iter().enumerate() {
					write!(f, "{}", ty)?;
					if i != struct_ty.len() - 1 {
						write!(f, ", ")?;
					}
				}
				write!(f, " \\}}")?;
				Ok(())
			}
			Type::Vector(vec) => write!(f, "[ {} x {} ]", vec.count, *vec.ty),
			Type::Void => write!(f, "void"),
			Type::Int(size) => write!(f, "i{}", size),
			_ => write!(f, "{:?}", self),
		}
	}
}
