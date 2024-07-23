use flux_span::{FileId, Interner, Span, WithSpan};
use flux_typesystem::{
    ConcreteKind::*, Generic, Insert, Path, TChecker, TraitRestriction, TypeId, TypeKind::*,
};

macro_rules! insert_basic_path {
    ($tchk:expr, $file_id:expr, $span:expr, $path:literal) => {
        $tchk.tenv.insert(
            Concrete(Path(Path::new(
                vec![$tchk.tenv.interner.get_or_intern($path)],
                vec![],
            )))
            .file_span($file_id, $span),
        )
    };
}

pub struct Primitives {
    file_id: FileId,
    span: Span,
}

impl Primitives {
    pub fn new(file_id: FileId) -> Self {
        Self {
            file_id,
            span: Span::poisoned(),
        }
    }

    pub fn u64(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "u64")
    }

    pub fn u32(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "u32")
    }

    pub fn u16(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "u16")
    }

    pub fn u8(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "u8")
    }

    pub fn s64(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "s64")
    }

    pub fn s32(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "s32")
    }

    pub fn s16(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "s16")
    }

    pub fn s8(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "s8")
    }

    pub fn f64(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "f64")
    }

    pub fn f32(&self, tchk: &mut TChecker) -> TypeId {
        insert_basic_path!(tchk, self.file_id, self.span, "f32")
    }

    pub fn int(&self, tchk: &mut TChecker, ref_to: Option<TypeId>) -> TypeId {
        tchk.tenv
            .insert(Int(ref_to).file_span(self.file_id, self.span))
    }

    pub fn float(&self, tchk: &mut TChecker, ref_to: Option<TypeId>) -> TypeId {
        tchk.tenv
            .insert(Float(ref_to).file_span(self.file_id, self.span))
    }

    pub fn never(&self, tchk: &mut TChecker) -> TypeId {
        tchk.tenv.insert(Never.file_span(self.file_id, self.span))
    }

    pub fn unknown(&self, tchk: &mut TChecker) -> TypeId {
        tchk.tenv.insert(Unknown.file_span(self.file_id, self.span))
    }

    pub fn tuple(&self, tchk: &mut TChecker, types: Vec<TypeId>) -> TypeId {
        tchk.tenv
            .insert(Concrete(Tuple(types)).file_span(self.file_id, self.span))
    }

    pub fn array(&self, tchk: &mut TChecker, ty: TypeId, n: u64) -> TypeId {
        tchk.tenv
            .insert(Concrete(Array(ty, n)).file_span(self.file_id, self.span))
    }

    pub fn ptr(&self, tchk: &mut TChecker, to: TypeId) -> TypeId {
        tchk.tenv
            .insert(Concrete(Ptr(to)).file_span(self.file_id, self.span))
    }
}

pub struct Traits {
    interner: &'static Interner,
    file_id: FileId,
    span: Span,
}

impl Traits {
    pub fn new(interner: &'static Interner, file_id: FileId, span: Span) -> Self {
        Self {
            interner,
            file_id,
            span,
        }
    }

    pub fn generic(
        &self,
        tchk: &mut TChecker,
        name: &str,
        restrictions: Vec<TraitRestriction>,
    ) -> TypeId {
        tchk.tenv.insert(
            Generic(Generic::new(
                self.interner.get_or_intern(name),
                restrictions,
            ))
            .file_span(self.file_id, self.span),
        )
    }

    pub fn restriction(&self, name: &str, args: Vec<TypeId>) -> TraitRestriction {
        TraitRestriction::new(vec![self.interner.get_or_intern(name)], args)
    }
}
