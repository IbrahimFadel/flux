use std::{marker::PhantomData, sync::Arc};

#[derive(Debug)]
pub enum Type {
    Array(TypeRef, u32),
    UIntTy(UIntTy),
    SIntTy(SIntTy),
    Ptr(TypeRef),
    Tuple(Vec<TypeRef>),
}

#[derive(Debug)]
pub enum UIntTy {
    U64,
    U32,
    U16,
    U8,
}

#[derive(Debug)]
pub enum SIntTy {
    S64,
    S32,
    S16,
    S8,
}

#[derive(Debug)]
pub struct TypeRef<P = ()> {
    ptr: Arc<Type>,
    variant: PhantomData<P>,
}

impl TypeRef {
    pub fn new(ptr: Arc<Type>) -> Self {
        Self {
            ptr,
            variant: PhantomData,
        }
    }
}
