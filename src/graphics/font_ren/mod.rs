use std::borrow::Cow;
use glium;
use glium::{Surface, BlendingFunction, LinearBlendingFactor};
use glium::texture::RawImage2d;
use rusttype::gpu_cache::Cache;
use rusttype::{FontCollection, Scale, Point, PositionedGlyph};
use glium::draw_parameters::{DrawParameters, Blend};
use vec::Vec2;

// Current strategy:
// every frame clear buffers and draw/upload everything again.

const MAX_NUM_VERTICES: usize = 8000;

pub struct FontRen<'a> {
    // Font/glyph management
    cache: Cache,
    fonts: FontCollection<'a>,

    // Pen position
    pen: Vec2,

    // OpenGL
    geometry: Vec<Vertex>,
    display: glium::Display,
    shader_prg: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    texture: glium::Texture2d,
}

const CACHE_SIZE: u32 = 512;

impl<'a> FontRen<'a> {
    pub fn new(display: glium::Display, fonts: FontCollection<'a>) -> FontRen<'a> {
        // OpenGL
        let vert_src = include_str!("../../../shaders/xyuv_tex.vert");
        let frag_src = include_str!("../../../shaders/xyuv_tex.frag");
        let shader_prg = glium::Program::from_source(&display, vert_src, frag_src, None).unwrap();
        let empty_texture: Vec<Vec<u8>> = vec!(vec!(0; CACHE_SIZE as usize); CACHE_SIZE as usize);
        let vertex_buffer = glium::VertexBuffer::empty(&display, MAX_NUM_VERTICES).unwrap(); // TODO dynamic?

        FontRen {
            cache: Cache::new(CACHE_SIZE, CACHE_SIZE, 0.2, 1.0),
            fonts: fonts,

            pen: Vec2::null_vec(),

            geometry: Vec::new(),
            display: display.clone(),
            shader_prg: shader_prg,
            vertex_buffer: vertex_buffer,
            texture: glium::texture::Texture2d::new(&display, empty_texture).unwrap(),
        }
    }
    pub fn clear(&mut self) {
        self.geometry.clear();
    }
    fn upload_vertices(&mut self) {
        if self.geometry.is_empty() {
            return;
        }
        let slice = self.vertex_buffer.slice(0..(self.geometry.len())).unwrap();
        slice.write(&self.geometry);
    }

    pub fn render(&mut self, target: &mut glium::Frame, center: (f32, f32), zoom: f32) {
        self.upload_vertices();
        let wsize = self.display.get_window().unwrap().get_inner_size().unwrap();
        let uniforms = uniform! (
            texture: glium::uniforms::Sampler::new(&self.texture)
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            proj: ortho_matrix(wsize.0 as f32, wsize.1 as f32, 0.0, 1.0),
            view: view_matrix(center, zoom, zoom),
            model: identity(),
        );
        let draw_params = DrawParameters {
            blend: Blend {
                color: BlendingFunction::Addition {
                    source: LinearBlendingFactor::SourceAlpha,
                    destination: LinearBlendingFactor::OneMinusSourceAlpha,
                },
                alpha: BlendingFunction::Max,
                .. Default::default()
            },
            .. Default::default()
        };
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        target.draw(self.vertex_buffer.slice(0..self.geometry.len()).unwrap(),
                  indices,
                  &self.shader_prg,
                  &uniforms,
                  &draw_params)
            .unwrap();
    }

    pub fn draw_text(&mut self, text: String, scale: Scale, offset: Point<f32>) {
        let font = self.fonts.clone().into_font().unwrap();
        let glyphs: Vec<PositionedGlyph> = font.layout(text.as_str(), scale, offset).collect();
        for g in glyphs.iter() {
            self.cache.queue_glyph(0, g.clone());
        }

        { // Cache the queued glyphs...
          // Reason for borrowing texture here: so that the closure doesn't borrow mut self
            let texture = &mut self.texture;
            self.cache.cache_queued(|rect, data| {
                let upload_area = glium::Rect {
                    left: rect.min.x,
                    bottom: rect.min.y,
                    width: rect.max.x - rect.min.x,
                    height: rect.max.y - rect.min.y,
                };
                let upload_data = RawImage2d {
                    data: Cow::Borrowed(data),
                    width: rect.max.x - rect.min.x,
                    height: rect.max.y - rect.min.y,
                    format: glium::texture::ClientFormat::U8,
                };
                texture.write(upload_area, upload_data);
            }).unwrap();
        }

        let glyphs: Vec<PositionedGlyph> = font.layout(text.as_str(), scale, offset).collect();
        for g in glyphs {
            self.draw_glyph(&g);
        }
    }
    fn draw_glyph(&mut self, g: &PositionedGlyph) {
        let uv_rect = self.cache.rect_for(0, g).unwrap();
        if let Some((uv, xy)) = uv_rect {
            println!("Draw_glyph: {}, {} & {}, {}", xy.min.x, xy.max.x, xy.min.y, xy.max.y);
            self.geometry.push(Vertex::new(xy.min.x as f32, -xy.min.y as f32, uv.min.x, uv.min.y));
            self.geometry.push(Vertex::new(xy.max.x as f32, -xy.min.y as f32, uv.max.x, uv.min.y));
            self.geometry.push(Vertex::new(xy.max.x as f32, -xy.max.y as f32, uv.max.x, uv.max.y));

            self.geometry.push(Vertex::new(xy.max.x as f32, -xy.max.y as f32, uv.max.x, uv.max.y));
            self.geometry.push(Vertex::new(xy.min.x as f32, -xy.max.y as f32, uv.min.x, uv.max.y));
            self.geometry.push(Vertex::new(xy.min.x as f32, -xy.min.y as f32, uv.min.x, uv.min.y));
        }
    }
}


implement_vertex!(Vertex, pos, texpos);

#[derive(Copy,Clone)]
struct Vertex {
    pos: [f32; 2],
    texpos: [f32; 2],
}
impl Vertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32) -> Vertex {
        Vertex {
            pos: [x, y],
            texpos: [u, v],
        }
    }
}




fn view_matrix(center: (f32, f32), scale_x: f32, scale_y: f32) -> [[f32; 4]; 4] {
    // data views the transpose of the actual matrix
    [[scale_x, 0.0, 0.0, 0.0],
     [0.0, scale_y, 0.0, 0.0],
     [0.0, 0.0, 1.0, 0.0],
     [-center.0 * scale_x, -center.1 * scale_y, 0.0, 1.0]]
}
fn ortho_matrix(width: f32, height: f32, far: f32, near: f32) -> [[f32; 4]; 4] {
    let width = width as f32;
    let height = height as f32;
    let far = far as f32;
    let near = near as f32;
    [[2.0 / width, 0.0, 0.0, 0.0],
     [0.0, 2.0 / height, 0.0, 0.0],
     [0.0, 0.0, -2.0 / (far - near), 0.0],
     [0.0, 0.0, -(far + near) / (far - near), 1.0]]
}

fn identity() -> [[f32; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0],
     [0.0, 1.0, 0.0, 0.0],
     [0.0, 0.0, 1.0, 0.0],
     [0.0, 0.0, 0.0, 1.0]]
}
