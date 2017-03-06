pub mod font_ren;

use self::font_ren::FontRen;
use err::*;

use pdf;
use pdf::file;
use pdf::Content;
use pdf::doc::{Object, Document};
use glium::{Display, Surface};
use rusttype::{FontCollection, Scale, Point};

use std::fs::File;
use std::io::Read;

const FONT_PATH: &'static str = "Roboto-Regular.ttf";

pub struct PdfRenderer<'a, 'b> {
    doc: &'a Document,
    display: Display,
    font_ren: FontRen<'b>,
    fonts: FontCollection<'b>,
}


impl<'a, 'b> PdfRenderer<'a, 'b> {
    pub fn new(display: Display, doc: &'a Document) -> Result<PdfRenderer<'a, 'b>> {
        let contents = file_to_vec(FONT_PATH).unwrap();
        let fonts = FontCollection::from_bytes(contents);
        let font = fonts.font_at(0).unwrap();
        let mut renderer = PdfRenderer {
            doc: doc,
            display: display.clone(),
            font_ren: FontRen::new(display.clone(), fonts.clone()),
            fonts: fonts,
        };
        // renderer.font_ren.draw_text("Test 123".to_string(), Scale::uniform(20.0), Point {x: 0.0, y: 0.0});

        renderer.init()?;

        Ok(renderer)
    }

    fn init(&mut self) -> Result<()> {
        // As a test, just add text of the first page once.

        let content = self.get_page_content(0)?;

        println!("{:?}", content);
        let pen = Point {x: 0.0, y: 0.0};
        for op in content.operations {
            println!("Op {}", op);
            if op.operator == "TJ" {
                let operands = op.operands[0].as_array()?;
                for obj in operands {
                    match obj {
                        &file::Object::String (ref s) => {
                            let s = String::from_utf8(s.clone())?;
                            println!("Draw string: {}", s);
                            self.font_ren.draw_text(s, Scale::uniform(20.0), pen);
                        },
                        &file::Object::Integer (n) => {
                        }
                        o => {
                            println!("Other object: {:?}", o);
                        }
                    }
                }
            }
        }

        // println!("Strings: {:?}",)
        Ok(())
    }

    pub fn get_page_content(&mut self, page_nr: i32) -> Result<pdf::Content> {
        let page = self.doc.get_page(page_nr)?;
        let content = page.get("Contents")?; // TODO why we have to use a let binding here?
        let content = match *content.inner() {
            file::Object::Stream (_) => content.as_stream()?,
            file::Object::Array (_) => content.as_array()?.get(0).as_stream()?,
            _ => bail!("Contents neither Stream nor Array."),
        };

        let content = Content::parse_from(&content.content).chain_err(|| "Parsing contents.")?;
        Ok(content)
    }

    pub fn render(&mut self, center: (f32, f32), zoom: f32) {
        let mut target = self.display.draw();
        target.clear_color(0.1, 0.1, 0.1, 1.0);
        self.font_ren.render(&mut target, center, zoom);
        target.finish().unwrap();
    }

}

fn file_to_vec(path: &str) -> Result<Box<[u8]>> {

    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    let _ = file.read_to_end(&mut buf)?;
    Ok(buf.into_boxed_slice())
}
