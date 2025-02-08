// Adapted from: https://github.com/tfachmann/typst-as-library/blob/main/src/lib.rs
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::Library;
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
}

/// Main interface that determines the environment for Typst.
pub struct TypstWrapperWorld {
    /// The content of a source.
    source: Source,

    /// The standard library.
    library: LazyHash<Library>,

    /// Datetime.
    time: time::OffsetDateTime,
}

impl TypstWrapperWorld {
    pub fn new(source: String) -> Self {
        Self {
            library: LazyHash::new(Library::default()),
            source: Source::detached(source),
            time: time::OffsetDateTime::now_utc(),
        }
    }
}

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
            todo!()
        }
    }

    /// Accessing a specified file (non-file).
    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        todo!()
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
