pub mod font_ren;

use pdf::reader::PdfReader;
use self::font_ren::FontRen;
use glium::{Display, Surface};

use rusttype::{FontCollection, Scale, Point};

use std::fs::File;
use std::io::{Read, Result};

const FONT_PATH: &'static str = "Roboto-Regular.ttf";

pub struct PdfRenderer<'a> {
    pdf: PdfReader,
    display: Display,
    font_ren: FontRen<'a>,
    fonts: FontCollection<'a>,
}


impl<'a> PdfRenderer<'a> {
    pub fn new(display: Display, pdf: PdfReader) -> PdfRenderer<'a> {
        let contents = file_to_vec(FONT_PATH).unwrap();
        let fonts = FontCollection::from_bytes(contents);
        let font = fonts.font_at(0).unwrap();
        let mut renderer = PdfRenderer {
            pdf: pdf,
            display: display.clone(),
            font_ren: FontRen::new(display.clone(), fonts.clone()),
            fonts: fonts,
        };
        renderer.font_ren.draw_text("Test 123".to_string(), Scale::uniform(20.0), Point {x: 0.0, y: 0.0});

        renderer

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
