use std::collections::HashMap;

use flux_span::Word;
use la_arena::Idx;

use crate::item_scope::ItemScope;

pub(crate) type ModuleId = Idx<ModuleData>;

#[derive(Debug, Default)]
pub(crate) struct ModuleData {
    parent: Option<ModuleId>,
    children: HashMap<Word, ModuleId>,
    scope: ItemScope,
}
