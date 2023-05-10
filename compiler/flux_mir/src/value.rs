use std::{marker::PhantomData, sync::Arc};

#[derive(Debug)]
pub enum Value {
    Block(Block),
    BinOp(BinOp),
    // Parameter(Parameter),
    // Tuple(Tuple),
    // Pi(Pi),
    Lambda(Lambda),
    Int(u64),
    Tuple(Vec<ValRef>),
    // UInt(UInt),
    // SInt(SInt),
}

#[derive(Debug)]
pub struct Block(Vec<ValRef>);

impl Block {
    pub fn new(values: Vec<ValRef>) -> Self {
        Self(values)
    }
}

#[derive(Debug)]
pub struct Lambda {}

#[derive(Debug)]
pub enum Op {
    Eq,
    Sum,
}

#[derive(Debug)]
pub struct BinOp {
    lhs: ValRef,
    op: Op,
    rhs: ValRef,
}

impl BinOp {
    pub fn new(lhs: ValRef, op: Op, rhs: ValRef) -> Self {
        Self { lhs, op, rhs }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ValRef<P = ()> {
    ptr: Arc<Value>,
    variant: PhantomData<P>,
}

impl<T> ValRef<T> {
    pub fn new(ptr: Arc<Value>) -> Self {
        Self {
            ptr,
            variant: PhantomData,
        }
    }
}

// impl<T> ValRef<T> {
//     pub fn new(v: Arc<Value>) -> Self {
//         Self {
//             ptr: v,
//             variant: PhantomData,
//         }
//     }
// }

// pub type TypeRef = ValRef<IsType>;

// #[derive(Debug)]
// pub struct IsType;

// pub type TyArr = ValArr<IsType>;

// #[derive(Debug)]
// pub struct ValArr<P = ()> {
//     arr: Vec<ValRef<P>>,
//     variant: PhantomData<P>,
// }

// impl<T> ValArr<T> {
//     pub fn new() -> Self {
//         Self {
//             arr: vec![],
//             variant: PhantomData,
//         }
//     }

//     pub fn push(&mut self, v: ValRef<T>) {
//         self.arr.push(v);
//     }
// }
