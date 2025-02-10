// Adapted from: https://github.com/tfachmann/typst-as-library/blob/main/src/lib.rs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use comemo::track;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};
use typst_kit::fonts::{FontSlot, Fonts};

/// This struct is needed so we can return a single value from the `lazy_static`
struct FontsHolder {
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
}

lazy_static::lazy_static! {
    static ref FONTS: FontsHolder = {
        // TODO: System fonts? Adds significant delay and may not be necessary.
        let fonts = Fonts::searcher().include_system_fonts(false).search();
        FontsHolder { book: fonts.book.into(), fonts: fonts.fonts }
    };

    static ref DEFAULT_WORLD: TypstWrapperWorld = TypstWrapperWorld::anon();
}

/// Main interface that determines the environment for Typst.
pub struct TypstWrapperWorld {
    /// The content of a source.
    source: Source,

    /// The standard library.
    pub(crate) library: LazyHash<Library>,

    /// Datetime.
    time: time::OffsetDateTime,

    /// Map of all known files.
    files: Arc<Mutex<HashMap<FileId, FileEntry>>>,
}

impl TypstWrapperWorld {
    pub fn anon() -> Self {
        Self {
            library: LazyHash::new(Library::default()),
            source: Source::detached(""),
            time: time::OffsetDateTime::now_utc(),
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new(source: impl Into<String>) -> Self {
        Self {
            library: LazyHash::new(Library::default()),
            source: Source::detached(source),
            time: time::OffsetDateTime::now_utc(),
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Helper to handle file requests.
    fn get_file(&self, id: FileId) -> FileResult<FileEntry> {
        let mut files = self.files.lock().map_err(|_| FileError::AccessDenied)?;
        if let Some(entry) = files.get(&id) {
            return Ok(entry.clone());
        }
        let path = if let Some(package) = id.package() {
            Err(typst::diag::PackageError::NotFound(package.clone()))?
        } else {
            id.vpath().resolve(&std::env::current_dir().unwrap())
        }
        .ok_or(FileError::AccessDenied)?;

        let content = std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
        Ok(files
            .entry(id)
            .or_insert(FileEntry::new(content, None))
            .clone())
    }
}

/// A File that will be stored in the HashMap.
#[derive(Clone, Debug)]
struct FileEntry {
    bytes: Bytes,
    source: Option<Source>,
}

impl FileEntry {
    fn new(bytes: Vec<u8>, source: Option<Source>) -> Self {
        Self {
            bytes: bytes.into(),
            source,
        }
    }

    fn source(&mut self, id: FileId) -> FileResult<Source> {
        let source = if let Some(source) = &self.source {
            source
        } else {
            let contents = std::str::from_utf8(&self.bytes).map_err(|_| FileError::InvalidUtf8)?;
            let contents = contents.trim_start_matches('\u{feff}');
            let source = Source::new(id, contents.into());
            self.source.insert(source)
        };
        Ok(source.clone())
    }
}

#[track]
impl typst::World for TypstWrapperWorld {
    /// Standard library.
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Metadata about all known Books.
    fn book(&self) -> &LazyHash<FontBook> {
        &FONTS.book
    }

    /// Accessing the main source file.
    fn main(&self) -> FileId {
        self.source.id()
    }

    /// Accessing a specified source file (based on `FileId`).
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            self.get_file(id)?.source(id)
        }
    }

    /// Accessing a specified file (non-file).
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.get_file(id).map(|file| file.bytes.clone())
    }

    /// Accessing a specified font per index of font book.
    fn font(&self, id: usize) -> Option<Font> {
        FONTS.fonts[id].get()
    }

    /// Get the current date.
    ///
    /// Optionally, an offset in hours is given.
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}
