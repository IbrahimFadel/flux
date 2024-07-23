use flux_diagnostics::{ice, Diagnostic};
use flux_span::{FileId, Interner, Spanned, ToSpan, WithSpan, Word};
use flux_syntax::ast::{self, AstNode};

use crate::{
    hir::{ArrayType, Generic, GenericParams, Path, ThisPath, Type, TypeBound, TypeBoundList},
    POISONED_NAME,
};

// Methods for lowering nodes needed in both `ItemTree` and body lowering stages
pub(crate) struct LoweringCtx {
    pub(crate) file_id: FileId,
    pub(crate) interner: &'static Interner,
    diagnostics: Vec<Diagnostic>,
}

impl LoweringCtx {
    pub(crate) fn new(file_id: FileId, interner: &'static Interner) -> Self {
        Self {
            file_id,
            interner,
            diagnostics: vec![],
        }
    }

    pub fn lower_node<N, T, P, F>(
        &mut self,
        node: Option<N>,
        poison_function: P,
        normal_function: F,
    ) -> T
    where
        N: AstNode,
        P: FnOnce(&mut Self, N) -> T,
        F: FnOnce(&mut Self, N) -> T,
    {
        let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
        if n.is_poisoned() {
            poison_function(self, n)
        } else {
            normal_function(self, n)
        }
    }

    pub fn lower_optional_node<N, T, P, F>(
        &mut self,
        node: Option<N>,
        poison_function: P,
        normal_function: F,
    ) -> T
    where
        N: AstNode,
        P: FnOnce(&mut Self) -> T,
        F: FnOnce(&mut Self, N) -> T,
    {
        match node {
            Some(n) => {
                if n.is_poisoned() {
                    poison_function(self)
                } else {
                    normal_function(self, n)
                }
            }
            None => poison_function(self),
        }
    }

    pub(crate) fn lower_name(&mut self, name: Option<ast::Name>) -> Spanned<Word> {
        self.lower_node(
            name,
            |this, name| {
                this.interner
                    .get_or_intern_static(POISONED_NAME)
                    .at(name.range().to_span())
            },
            |_, name| {
                let name = name
                    .ident()
                    .unwrap_or_else(|| ice("name parsed without identifier token"));
                let key = name.text_key().unwrap_or_else(|| ice("parsed empty name"));
                key.at(name.text_range().to_span())
            },
        )
    }

    pub(crate) fn lower_path(
        &mut self,
        path: Option<ast::Path>,
        generic_params: &GenericParams,
    ) -> Spanned<Path> {
        self.lower_node(
            path,
            |_, path| Path::poisoned().at(path.range().to_span()),
            |this, path| {
                let segments = path
                    .segments()
                    .map(|segment| {
                        segment
                            .text_key()
                            .unwrap_or_else(|| ice("text key contained no text"))
                    })
                    .collect();
                let generic_args = path
                    .generic_arg_list()
                    .map(|arg_list| {
                        arg_list
                            .args()
                            .map(|arg| this.lower_type(Some(arg), generic_params).inner)
                            .collect()
                    })
                    .unwrap_or(vec![]);
                Path::new(segments, generic_args).at(path.range().to_span())
            },
        )
    }

    pub(crate) fn lower_type(
        &mut self,
        ty: Option<ast::Type>,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        self.lower_node(
            ty,
            |_, ty| Type::Unknown.at(ty.range().to_span()),
            |this, ty| match ty {
                ast::Type::PathType(path_type) => this.lower_path_type(path_type, generic_params),
                ast::Type::TupleType(tuple_type) => {
                    this.lower_tuple_type(tuple_type, generic_params)
                }
                ast::Type::ArrayType(arr_type) => this.lower_arr_type(arr_type, generic_params),
                ast::Type::PtrType(ptr_type) => this.lower_ptr_type(ptr_type, generic_params),
                ast::Type::ThisPathType(this_path_type) => {
                    this.lower_this_path_type(this_path_type, generic_params)
                }
            },
        )
    }

    pub(super) fn lower_path_type(
        &mut self,
        path_type: ast::PathType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let path = self.lower_path(path_type.path(), generic_params);
        let ty = if path.is_generic(generic_params) {
            let name = path.map(|path| *path.get(0));
            let key = name.inner;
            Type::Generic(Generic::new(
                name,
                get_trait_restrictions_on_generic(&key, generic_params),
            ))
        } else {
            Type::Path(Path::new(path.segments.clone(), vec![]))
        };
        ty.at(path_type.range().to_span())
    }

    fn lower_this_path_type(
        &mut self,
        this_path_type: ast::ThisPathType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let path = self.lower_path(this_path_type.path(), generic_params);
        path.map(|path| Type::ThisPath(ThisPath::new(path)))
    }

    fn lower_tuple_type(
        &mut self,
        tuple_type: ast::TupleType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let types = tuple_type
            .types()
            .map(|ty| self.lower_type(Some(ty), generic_params).inner)
            .collect();
        Type::Tuple(types).at(tuple_type.range().to_span())
    }

    fn lower_arr_type(
        &mut self,
        arr_type: ast::ArrayType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let ty = self.lower_type(arr_type.ty(), generic_params);
        tracing::warn!("hehehehhe unwrap go brr");
        let n: u64 = self
            .interner
            .resolve(&arr_type.n().unwrap().v().unwrap().text_key().unwrap())
            .parse()
            .unwrap();
        Type::Array(ArrayType::new(ty.inner, n)).at(arr_type.range().to_span())
    }

    fn lower_ptr_type(
        &mut self,
        ptr_type: ast::PtrType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let to_ty = self.lower_type(ptr_type.ty(), generic_params);
        Type::Ptr(Box::new(to_ty.inner)).at(ptr_type.range().to_span())
    }
}

fn get_trait_restrictions_on_generic(name: &Word, generic_params: &GenericParams) -> TypeBoundList {
    let bounds = generic_params
        .where_predicates
        .iter()
        .filter_map(|predicate| {
            if predicate.name == *name {
                Some(
                    predicate
                        .bound
                        .map_ref(|bound| TypeBound::new(bound.clone())),
                )
            } else {
                None
            }
        });
    TypeBoundList::new(bounds.collect())
}
