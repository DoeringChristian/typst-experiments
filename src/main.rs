use chrono::{DateTime, Datelike, Local};
use comemo::Prehashed;
use tera::Tera;
use typst::diag::{FileError, FileResult};
use typst::eval::Tracer;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook, FontFamily};
use typst::{Library, World};

const FONTS: &[&[u8]] = &[include_bytes!("../assets/fonts/NewCM10-Regular.otf")];
const LIBRARIES: &[(&str, &str)] = &[];

pub struct TypstWorld {
    library: Prehashed<Library>,
    font_book: Prehashed<FontBook>,
    fonts: Vec<Font>,
    source: Source,
    date: DateTime<Local>,
    ext_libraries: Vec<Source>,
}

impl TypstWorld {
    pub fn new(source: String, fonts: &[&[u8]], ext_libraries: &[(&str, &str)]) -> Self {
        let fonts: Vec<Font> = fonts
            .iter()
            .map(|f| Font::new(Bytes::from(*f), 0).unwrap())
            .collect();

        let ext_libraries: Vec<Source> = ext_libraries
            .iter()
            .map(|(p, f)| Source::new(FileId::new(None, VirtualPath::new(p)), String::from(*f)))
            .collect();

        let library = Library::builder().build();

        Self {
            library: Prehashed::new(library),
            font_book: Prehashed::new(FontBook::from_fonts(fonts.iter())),
            fonts,
            source: Source::new(FileId::new(None, VirtualPath::new("/main.typ")), source),
            date: Local::now(),
            ext_libraries,
        }
    }
}

impl World for TypstWorld {
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.font_book
    }

    fn main(&self) -> Source {
        self.source.clone()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else if self.ext_libraries.iter().find(|l| id == l.id()).is_some() {
            Ok(self
                .ext_libraries
                .iter()
                .find(|l| id == l.id())
                .unwrap()
                .clone())
        } else {
            Err(FileError::NotFound(
                id.vpath().as_rooted_path().to_path_buf(),
            ))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(
            id.vpath().as_rooted_path().to_path_buf(),
        ))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).map(|f| f.clone())
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let date = if let Some(offset) = offset {
            self.date.naive_utc() + chrono::Duration::try_hours(offset)?
        } else {
            self.date.naive_local()
        };

        Datetime::from_ymd(
            date.year(),
            date.month().try_into().ok()?,
            date.day().try_into().ok()?,
        )
    }
}

fn main() {
    let template = include_str!("../assets/templates/test01.typ");

    let mut tera = Tera::new("assets/templates/**.typ").unwrap();
    tera.add_raw_template("template", template).unwrap();

    let mut context = tera::Context::new();

    let rendered = tera.render("template", &context).unwrap();

    let world = TypstWorld::new(rendered, FONTS, LIBRARIES);

    let doc = typst::compile(&world, &mut Tracer::new()).unwrap();
    let pdf = typst_pdf::pdf(&doc, None, world.today(Some(0)));

    std::fs::write("out/test.pdf", pdf).unwrap();
}
