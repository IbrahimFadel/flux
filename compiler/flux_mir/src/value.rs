use std::{marker::PhantomData, sync::Arc};

pub enum Value {
    // Parameter(Parameter),
    // Tuple(Tuple),
    // Pi(Pi),
    // Lambda(Lambda),
    UInt(UInt),
    SInt(SInt),
    BinOp(BinOp),
}

enum UInt {
    U64,
    U32,
    U16,
    U8,
}

enum SInt {
    S64,
    S32,
    S16,
    S8,
}

pub enum Op {}

pub struct BinOp {
    lhs: ValRef,
    op: Op,
    rhs: ValRef,
}

#[repr(transparent)]
pub struct ValRef<P = ()> {
    ptr: Arc<Value>,
    variant: PhantomData<P>,
}

pub type TypeRef = ValRef<IsType>;

pub struct IsType;

pub type TyArr = ValArr<IsType>;

pub struct ValArr<P = ()> {
    arr: Vec<ValRef<P>>,
    variant: PhantomData<P>,
}
