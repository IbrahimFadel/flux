#[salsa::interned]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Word {
    #[return_ref]
    pub string: String,
}

impl Word {
    pub fn intern(db: &dyn crate::Db, string: impl ToString) -> Self {
        Word::new(db, string.to_string())
    }

    pub fn as_str(self, db: &dyn crate::Db) -> &str {
        self.string(db)
    }

    pub fn to_string(self, db: &dyn crate::Db) -> String {
        self.string(db).clone()
    }
}
