use std::marker::PhantomData;

use flux_span::InFile;
use flux_syntax::ast::AstNode;
use la_arena::Idx;

pub(crate) type AstId<N> = InFile<FileAstId<N>>;

#[derive(Debug)]
pub struct FileAstId<N: AstNode> {
    raw: ErasedFileAstId,
    _ty: PhantomData<fn() -> N>,
}

type ErasedFileAstId = Idx<Synt>;
