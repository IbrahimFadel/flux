use flux_diagnostics::{Diagnostic, SourceCache, ToDiagnostic};
use flux_span::{FileId, FileSpanned};

use crate::diagnostics::LowerError;

pub(crate) mod diagnostics;
pub(crate) mod import;
pub(crate) mod item;

const MOD_DEPTH_LIMIT: u32 = 16;

#[derive(Debug)]
pub(crate) struct ModDir {
    dir_path: DirPath,
    depth: u32,
}

impl ModDir {
    pub(super) fn root() -> ModDir {
        ModDir {
            dir_path: DirPath::empty(),
            depth: 0,
        }
    }

    pub(super) fn prelude() -> ModDir {
        let mut dir_path = DirPath::empty();
        dir_path.push("prelude");
        ModDir { dir_path, depth: 1 }
    }

    fn child(&self, dir_path: DirPath) -> Option<ModDir> {
        let depth = self.depth + 1;
        if depth == MOD_DEPTH_LIMIT {
            tracing::error!("MOD_DEPTH_LIMIT exceeded {:?}", dir_path);
            return None;
        }
        Some(ModDir { dir_path, depth })
    }

    pub(super) fn resolve_declaration<R: FileResolver>(
        &self,
        file_id: FileId,
        name: &FileSpanned<&str>,
        source_cache: &mut SourceCache,
        resolver: &R,
    ) -> Result<(FileId, String, ModDir), Diagnostic> {
        let candidate_files = &[
            format!("{}{}.flx", self.dir_path.0, name.inner.inner),
            format!(
                "{}{}/{}.flx",
                self.dir_path.0, name.inner.inner, name.inner.inner
            ),
        ];

        for candidate in candidate_files.iter() {
            let path = RelativePath {
                anchor: file_id,
                path: candidate.as_str(),
            };
            if let Some((file_id, content)) = resolver.resolve_relative_path(path, source_cache) {
                let is_mod_flx = candidate.ends_with(&format!("/{}.flx", name.inner.inner));

                let dir_path = if is_mod_flx {
                    DirPath::empty()
                } else {
                    DirPath::new(format!("{}/", name.inner.inner))
                };
                if let Some(mod_dir) = self.child(dir_path) {
                    return Ok((file_id, content, mod_dir));
                }
            }
        }
        Err(LowerError::CouldNotResolveModDecl {
            decl: name.inner.inner.to_string(),
            decl_file_span: name.to_file_span(),
            candidate_paths: candidate_files.to_vec(),
        }
        .to_diagnostic())
    }
}

#[derive(Clone, Debug)]
struct DirPath(String);

impl DirPath {
    #[inline]
    fn assert_invariant(&self) {
        assert!(self.0.is_empty() || self.0.ends_with('/'));
    }

    fn new(repr: String) -> DirPath {
        let res = DirPath(repr);
        res.assert_invariant();
        res
    }

    fn empty() -> DirPath {
        DirPath::new(String::new())
    }

    fn push(&mut self, name: &str) {
        self.0.push_str(name);
        self.0.push('/');
        self.assert_invariant();
    }

    // fn parent(&self) -> Option<&str> {
    //     if self.0.is_empty() {
    //         return None;
    //     };
    //     let idx = self.0[..self.0.len() - '/'.len_utf8()]
    //         .rfind('/')
    //         .map_or(0, |it| it + '/'.len_utf8());
    //     Some(&self.0[..idx])
    // }
}

/// Path relative to a file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RelativePath<'a> {
    /// File that this path is relative to.
    pub anchor: FileId,
    /// Path relative to `anchor`'s containing directory.
    pub path: &'a str,
}

pub trait FileResolver {
    fn resolve_absolute_path(
        &self,
        path: &str,
        source_cache: &mut SourceCache,
    ) -> Option<(FileId, String)> {
        std::fs::read_to_string(path)
            .map(|result| {
                let file_id = source_cache.add_input_file(path, result.clone());
                (file_id, result)
            })
            .ok()
    }

    fn resolve_relative_path(
        &self,
        path: RelativePath,
        source_cache: &mut SourceCache,
    ) -> Option<(FileId, String)> {
        let anchor_path = source_cache.get_file_dir(&path.anchor);
        let absolute_path = format!("{anchor_path}/{}", path.path);
        match std::fs::read_to_string(&absolute_path) {
            Ok(result) => {
                let file_id = source_cache.add_input_file(&absolute_path, result.clone());
                Some((file_id, result))
            }
            Err(_) => None,
        }
    }
}

pub struct BasicFileResolver;

impl FileResolver for BasicFileResolver {}
