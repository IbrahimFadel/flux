use std::{collections::HashMap, rc::Rc};

pub type NodeId = usize;
pub type PortId = usize;
pub type ValId = usize;

#[derive(Debug)]
pub struct Rvsdg {
	pub(crate) nodes: HashMap<NodeId, Node>,
	ports: HashMap<PortId, PortData>,
	edges: HashMap<OutputPort, (InputPort, EdgeKind)>,
}

impl Rvsdg {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
			ports: HashMap::new(),
			edges: HashMap::new(),
		}
	}

	pub fn add_edge(&mut self, from: OutputPort, to: InputPort, kind: EdgeKind) {
		self.edges.insert(from, (to, kind));
	}

	pub fn add_node(&mut self, id: NodeId, node: Node) {
		self.nodes.insert(id, node);
	}

	pub fn next_node(&self) -> NodeId {
		self.nodes.len()
	}

	pub fn create_input_port(&mut self, parent: NodeId, edge: EdgeKind, ty: Type) -> InputPort {
		let port = self.next_port();
		self.ports.insert(port, PortData::input(parent, edge));
		InputPort::new(port, ty)
	}

	pub fn create_value_input(&mut self, parent: NodeId, ty: Type) -> InputPort {
		self.create_input_port(parent, EdgeKind::Value, ty)
	}

	pub fn create_state_input(&mut self, parent: NodeId) -> InputPort {
		self.create_input_port(parent, EdgeKind::State, Type::Unit)
	}

	pub fn create_output_port(&mut self, parent: NodeId, edge: EdgeKind) -> OutputPort {
		let port = self.next_port();
		self.ports.insert(port, PortData::output(parent, edge));
		OutputPort::new(port, Type::Unit)
	}

	pub fn create_value_output(&mut self, parent: NodeId) -> OutputPort {
		self.create_output_port(parent, EdgeKind::Value)
	}

	pub fn create_effect_output(&mut self, parent: NodeId) -> OutputPort {
		self.create_output_port(parent, EdgeKind::State)
	}

	#[inline]
	fn next_port(&mut self) -> PortId {
		self.ports.len()
	}
}

#[derive(Debug, Clone)]
pub struct Omega {
	pub(crate) node: NodeId,
	pub(crate) state: OutputPort,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputPort {
	id: PortId,
	ty: Type,
}

impl OutputPort {
	pub fn new(id: PortId, ty: Type) -> Self {
		Self { id, ty }
	}
}

#[derive(Debug, Clone)]
pub struct InputPort {
	id: PortId,
	ty: Type,
}

impl InputPort {
	pub fn new(id: PortId, ty: Type) -> Self {
		Self { id, ty }
	}
}

#[derive(Debug)]
pub enum PortKind {
	Input,
	Output,
}

#[derive(Debug)]
pub struct PortData {
	pub kind: PortKind,
	pub parent: NodeId,
	pub edge: EdgeKind,
}

impl PortData {
	pub fn input(parent: NodeId, edge: EdgeKind) -> Self {
		Self {
			kind: PortKind::Input,
			parent,
			edge,
		}
	}

	pub fn output(parent: NodeId, edge: EdgeKind) -> Self {
		Self {
			kind: PortKind::Output,
			parent,
			edge,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
	Int(u32),
	Unit,
}

/// Wrapper to RegionData
/// An Omega node can be represented as a Region that hold the None value
#[derive(Debug, Clone)]
pub struct Region(Option<Rc<RegionData>>);

impl Region {
	pub fn omega() -> Self {
		Self(None)
	}
}

/// A region contains arguments, nodes, edges, and results.
#[derive(Debug, Clone)]
pub struct RegionData {
	arguments: Vec<ValId>,
	nodes: Vec<Node>,
	edges: Vec<Edge>,
	results: Vec<ValId>,
}

#[derive(Debug, Clone)]
pub enum Node {
	Omega(Omega),
	Lambda(Lambda),
}

#[derive(Debug, Clone)]
pub struct Lambda {
	id: NodeId,
	inputs: Vec<InputPort>,
	output: OutputPort,
	region: Region,
}

impl Lambda {
	pub fn new(id: NodeId, inputs: Vec<InputPort>, output: OutputPort, region: Region) -> Self {
		Self {
			id,
			inputs,
			output,
			region,
		}
	}
}

#[derive(Debug, Clone)]
pub enum EdgeKind {
	Value,
	State,
}

#[derive(Debug, Clone)]
pub struct Edge {
	kind: EdgeKind,
}

// pub struct Region(Option<Rc<RegionData>>);

// impl Region {
// 	pub fn omega() -> Self {
// 		Self(None)
// 	}
// }

// pub struct RegionData {
// 	parents: Vec<Region>,
// 	inputs: Vec<Value>,
// }

// pub struct Lambda {
// 	inputs: Vec<ValId>,
// 	output: ValId,
// 	// ty: Type,
// 	region: Region,
// }

// pub enum Value {}

// use std::{cell::RefCell, collections::HashMap, rc::Rc};

// pub type NodeId = usize;
// pub type VarId = usize;

// type Outputs = HashMap<VarId, Value>;
// type Inputs = HashMap<VarId, Value>;

// pub struct RVSDG {
// 	omega: Omega,
// }

// impl RVSDG {
// 	pub fn new() -> Self {
// 		Self {
// 			omega: Omega::new(),
// 		}
// 	}
// }

// #[derive(Debug, Clone)]
// pub struct Region {
// 	nodes: Vec<Node>,
// 	edges: Vec<Edge>,
// 	pub values: Vec<Value>,
// }

// impl Region {
// 	pub fn new() -> Self {
// 		Self {
// 			nodes: vec![],
// 			edges: vec![],
// 			values: vec![],
// 		}
// 	}
// }

// #[derive(Debug, Clone)]
// pub enum EdgeKind {
// 	Effect,
// 	Value,
// }

// #[derive(Debug, Clone)]
// pub struct Edge {
// 	kind: EdgeKind,
// 	indices: (usize, usize),
// }

// /// "An ω-node models a translation unit. It is the top-level node of an RVSDG and has no inputs or outputs.
// /// It contains exactly one region."
// pub struct Omega {
// 	pub region: Rc<RefCell<Region>>,
// }

// impl Omega {
// 	pub fn new() -> Self {
// 		Self {
// 			region: Rc::new(RefCell::new(Region::new())),
// 		}
// 	}
// }

// #[derive(Debug, Clone)]
// pub enum Value {
// 	Var(VarId),
// 	Const(Const),
// 	Missing,
// }

// #[derive(Debug, Clone)]
// pub enum Const {
// 	// Ptr(Ptr),
// 	// Cell(Cell),
// 	// Bool(bool),
// }

// #[derive(Debug, Clone)]
// pub enum Expr {
// 	// Cmp(Cmp),
// 	// Add(Add),
// 	// Sub(Sub),
// 	// Mul(Mul),
// 	// Not(Not),
// 	// Neg(Neg),
// 	// Load(Load),
// 	Apply(Apply),
// 	Value(Value),
// }

// #[derive(Debug, Clone)]
// pub struct Block {
// 	instructions: Vec<Node>,
// }

// #[derive(Debug, Clone)]
// pub enum Node {
// 	Apply(Apply),
// 	Assign(Assign),
// 	Store(Store),
// 	Gamma(Gamma),
// 	Theta(Theta),
// 	Lambda(Lambda),
// }

// #[derive(Debug, Clone)]
// pub struct Apply {
// 	pub node: NodeId,
// 	pub args: Vec<Value>,
// }

// #[derive(Debug, Clone)]
// pub struct Assign {
// 	pub var: VarId,
// 	pub val: Expr,
// }

// #[derive(Debug, Clone)]
// pub struct Store {
// 	pub ptr: Value,
// 	pub value: Value,
// }

// #[derive(Debug, Clone)]
// pub struct Gamma {
// 	pub node: NodeId,
// 	pub cond: Value,
// 	pub true_block: Block,
// 	pub false_block: Block,
// 	pub true_outputs: Outputs,
// 	pub false_outputs: Outputs,
// }

// #[derive(Debug, Clone)]
// pub struct Theta {
// 	pub node: NodeId,
// 	pub body: Block,
// 	pub cond: Value,
// 	pub inputs: Inputs,
// 	pub outputs: Outputs,
// }

// #[derive(Debug, Clone)]
// pub struct Lambda {
// 	inputs: Inputs,
// 	output: Value,
// 	pub region: Rc<RefCell<Region>>,
// }

// impl Lambda {
// 	pub fn new() -> Self {
// 		Self {
// 			inputs: HashMap::new(),
// 			output: Value::Missing,
// 			region: Rc::new(RefCell::new(Region::new())),
// 		}
// 	}
// }
