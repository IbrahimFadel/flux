// #[derive(Debug, Clone)]
// pub enum Type {
// 	I64,
// 	U64,
// 	I32,
// 	U32,
// 	I16,
// 	U16,
// 	I8,
// 	U8,
// 	F64,
// 	F32,
// 	Bool,
// 	Void,
// }

// #[derive(Debug)]
// pub struct FnDecl {
// 	type_: Type,
// 	name: &'static str,
// 	block: BasicBlock,
// }

// #[derive(Debug)]
// pub enum Instruction {
// 	StackAlloc(Type),
// 	HeapAlloc,
// 	Assign(Assign),
// }

// #[derive(Debug)]
// pub struct Assign {
// 	lhs: Box<Local>,
// 	rhs: RValue,
// }

// impl Assign {
// 	pub fn new(lhs: Box<Local>, rhs: RValue) -> Self {
// 		Self { lhs, rhs }
// 	}
// }

// #[derive(Debug)]
// pub enum Terminator {
// 	Return,
// 	Goto,
// 	// Call
// }

// #[derive(Debug)]
// pub enum RValue {
// 	BinOp,
// 	UnaryOp,
// 	I32(i32),
// }

// pub type Local = usize;

// #[derive(Debug)]
// pub struct BasicBlock {
// 	statements: Vec<Instruction>,
// 	terminator: Terminator,
// }
