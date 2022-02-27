use codespan_reporting::files;
use std::ops::Range;

/// A file that is backed by an `Arc<String>`.
#[derive(Debug, Clone)]
struct File {
    /// The name of the file.
    name: String,
    /// The source code of the file.
    source: String,
    /// The starting byte indices in the source code.
    line_starts: Vec<usize>,
}

impl File {
    fn line_start(&self, line_index: usize) -> Result<usize, files::Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Ok(*self
                .line_starts
                .get(line_index)
                .expect("failed despite previous check")),
            Ordering::Equal => Ok(self.source.len()),
            Ordering::Greater => Err(files::Error::LineTooLarge {
                given: line_index,
                max: self.line_starts.len() - 1,
            }),
        }
    }
}

/// An opaque file identifier.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FileId(u32);

#[derive(Debug, Clone)]
pub struct Files {
    files: Vec<File>,
}

impl Files {
    /// Create a new files database.
    pub fn new() -> Files {
        Files { files: Vec::new() }
    }

    /// Add a file to the database, returning the handle that can be used to
    /// refer to it again.
    pub fn add(&mut self, name: impl Into<String>, source: impl Into<String>) -> Option<FileId> {
        let file_id = FileId(u32::try_from(self.files.len()).ok()?);
        let name = name.into();
        let source = source.into();
        let line_starts = files::line_starts(&source).collect();

        self.files.push(File {
            name,
            line_starts,
            source,
        });

        Some(file_id)
    }

    /// Get the file corresponding to the given id.
    fn get(&self, file_id: FileId) -> Result<&File, files::Error> {
        self.files
            .get(file_id.0 as usize)
            .ok_or(files::Error::FileMissing)
    }
}

impl<'files> files::Files<'files> for Files {
    type FileId = FileId;
    type Name = &'files str;
    type Source = &'files str;

    fn name(&self, file_id: FileId) -> Result<&str, files::Error> {
        Ok(self.get(file_id)?.name.as_ref())
    }

    fn source(&self, file_id: FileId) -> Result<&str, files::Error> {
        Ok(&self.get(file_id)?.source)
    }

    fn line_index(&self, file_id: FileId, byte_index: usize) -> Result<usize, files::Error> {
        self.get(file_id)?
            .line_starts
            .binary_search(&byte_index)
            .or_else(|next_line| Ok(next_line - 1))
    }

    fn line_range(&self, file_id: FileId, line_index: usize) -> Result<Range<usize>, files::Error> {
        let file = self.get(file_id)?;
        let line_start = file.line_start(line_index)?;
        let next_line_start = file.line_start(line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}
