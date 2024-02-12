use std::collections::HashMap;

use flux_span::Word;

use crate::{hir::Visibility, item::ItemId};

#[derive(Debug, Default)]
pub(crate) struct ItemScope {
    functions: HashMap<Word, (Visibility, ItemId)>,
}
