extern crate rusttype;
#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate pdf;

pub mod graphics;
pub mod vec;

use pdf::reader::PdfReader;
use graphics::PdfRenderer;
use glium::{DisplayBuild, glutin};
use glium::glutin::{Event, MouseButton, ElementState, MouseScrollDelta};
use vec::Vec2;

const PATH: &'static str = "example.pdf";

// TODO NOW
// We need to be able to read stream (with graphics & text) before anything.


fn main() {
    let mut program = Program::new();
    program.run();
}


struct Program {
    center: Vec2,
    zoom: f32,

    mouse_down: bool,
    mouse_pos: Vec2,
    mouse_pos_past: Vec2,
}

impl Program {
    pub fn new() -> Program {
        Program {
            center: Vec2::null_vec(),
            zoom: 1.0,
            mouse_down: false,
            mouse_pos: Vec2::null_vec(),
            mouse_pos_past: Vec2::null_vec(),
        }
    }
    pub fn run(&mut self) {
        let display = glutin::WindowBuilder::new().build_glium().unwrap();
        let mut renderer = PdfRenderer::new(display.clone(), PdfReader::new(PATH).unwrap());
        loop {
            for ev in display.poll_events() {
                match ev {
                    Event::Closed => return,   // the window has been closed by the user
                    Event::MouseMoved(x, y) => self.mouse_moved(x, y),
                    Event::MouseWheel(MouseScrollDelta::LineDelta(_, y), _) => {
                        // self.mouse_wheel_line(y)
                    }
                    Event::MouseInput(ElementState::Pressed, button) => {
                        self.mouse_press(button)
                    }
                    Event::MouseInput(ElementState::Released, button) => {
                        self.mouse_release(button)
                    }
                    _ => ()
                }
            }
            renderer.render((self.center.x, self.center.y), self.zoom);
        }
    }

    fn mouse_moved(&mut self, x: i32, y: i32) {
        self.mouse_pos_past = self.mouse_pos;
        self.mouse_pos = Vec2::new(x as f32, y as f32);
        // Move the texture //
        if self.mouse_down {
            // let window_size = self.display.get_window().unwrap().get_inner_size().unwrap();
            let mut offset = (self.mouse_pos - self.mouse_pos_past) / self.zoom;
            offset.x = -offset.x;
            offset.y = offset.y;
            self.center += offset;
        }
    }

    fn mouse_wheel_line(&mut self, y: f32) {
        // For each 'tick', it should *= factor
        const ZOOM_FACTOR: f32 = 1.2;
        if y > 0.0 {
            self.zoom *= f32::powf(ZOOM_FACTOR, y as f32);
        } else if y < 0.0 {
            self.zoom /= f32::powf(ZOOM_FACTOR, -y as f32);
        }
    }

    fn mouse_press(&mut self, button: MouseButton) {
        if let MouseButton::Left = button {
            self.mouse_down = true;
        }
    }

    fn mouse_release(&mut self, button: MouseButton) {
        if let MouseButton::Left = button {
            self.mouse_down = false;
        }
    }
}
