// type RegionID = u32;
// type NodeID = u32;

// struct Region {
// 	id: RegionID,
// 	args: Vec<Value>,
// 	outputs: Vec<Value>,
// 	nodes: Vec<Node>,
// }

// struct Value {}

// struct Node {
// 	id: NodeID,
// 	data: NodeKind,
// }

// enum NodeKind {
// 	Simple(Simple), // + - * / etc.
// 	Gamma,          // if
// 	Theta,          // tail controlled loop
// 	Lambda,         // functions
// 	Delta,          // global variables
// 	Phi,            // recursive functions
// 	Omega,          // translation units
// }

// struct Simple {
// 	op: OpKind,
// }

// enum OpKind {
// 	Add,
// 	Sub,
// }
