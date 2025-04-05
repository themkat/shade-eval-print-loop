use std::{collections::HashMap, env::args, fs, path::Path, sync::mpsc::{channel, Receiver}};

use geometry::{SQUARE, Vertex};
use glium::{
    backend::{glutin::SimpleWindowBuilder, Facade}, glutin::surface::WindowSurface, index::NoIndices, uniforms::DynamicUniforms, winit::{application::ApplicationHandler, event_loop::EventLoop, window::Window}, Display, DrawParameters, Program, ProgramCreationError::CompilationError, Surface, VertexBuffer
};
use notify::{Event, Watcher};
use scheme::NetworkScheme;

mod geometry;
mod scheme;
mod command;

const VERTEX_SHADER: &str = "#version 330 core

in vec2 position;

void main() {
  gl_Position = vec4(position, 0.0, 1.0);
}";

const PLACEHOLDER_FRAGMENT_SHADER: &str = "#version 330 core

out vec4 color;

void main() {
  color = vec4(1.0, 0.8, 0.9, 1.0);
}
";

pub fn init() {
    // TODO: make it more clear with error messages etc. that program requires an input file
    let input_file = args().nth(1).unwrap_or("shaders/pass.frag".to_string());

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = SEPLApp::new(&event_loop, input_file);

    NetworkScheme::main_loop();
    event_loop.run_app(&mut app).expect("Could not run app");
}

struct SEPLApp {
    display: Display<WindowSurface>,
    window: Window,
    // TODO: option last error?
    last_error: String,
    input_file: String,
    input_file_watcher: Box<dyn Watcher>,
    input_file_events: Receiver<Result<Event, notify::Error>>,
    state: GLState,
}

struct GLState {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: NoIndices,
    program: Program,
    uniforms: DynamicUniforms<'static, 'static>,
}

impl SEPLApp {
    fn new(event_loop: &EventLoop<()>, fragment_shader_file: String) -> Self {
        let (window, display) = SimpleWindowBuilder::new()
            .with_inner_size(1280, 720)
            .with_title("Shade Eval Print Loop")
            .build(event_loop);

        let vertex_buffer =
            VertexBuffer::new(&display, &SQUARE).expect("Could not create vertex buffer");
        let index_buffer = NoIndices(glium::index::PrimitiveType::TriangleStrip);

        // listen to changes on the input file
        let (sender, receiver) = channel();
        // TODO: do we need to keep this around? or is it enough to just keep the receiver channel? Play around with it
        let mut input_file_watcher = notify::recommended_watcher(sender).expect("Could not initialize file watcher");
        input_file_watcher.watch(Path::new(fragment_shader_file.as_str()), notify::RecursiveMode::NonRecursive).expect("Could not create file watcher");

        // fallback initially to a placeholder if compilation error
        let mut program = Self::create_program(&display, fragment_shader_file.as_str());
        let mut last_error = String::new();
        if let Err(err) = program {
            last_error = err;            
            program = Program::from_source(&display, VERTEX_SHADER, PLACEHOLDER_FRAGMENT_SHADER, None).map_err(|_| "placeholder".to_string());
        }

        Self {
            window,
            display,
            last_error,
            input_file: fragment_shader_file,
            input_file_events: receiver,
            input_file_watcher: Box::new(input_file_watcher),
            state: GLState {
                vertex_buffer,
                index_buffer,
                program: program.expect("If this fails, it will be the end of Europe as we know it"),
                uniforms: DynamicUniforms::new(),
            },
        }
    }

    /// Read fragment shader from file, and create shader program combination. In our simplified scenario, the only reasonable error is a compilation error, so our error type is simply a String. 
    fn create_program<F: Facade>(display: &F, filename: &str) -> Result<Program, String> {
        let fragment_shader =
            fs::read_to_string(filename).expect("Could not read fragment shader!");

        Program::from_source(display, VERTEX_SHADER, fragment_shader.as_str(), None).map_err(|err| {
            if let CompilationError(compile_error, _) = err {
                compile_error
            } else {
                "POSSIBLE DRIVER ISSUE!".to_string()
            }
        })
    }
}

impl ApplicationHandler for SEPLApp {
    fn resumed(&mut self, _event_loop: &glium::winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &glium::winit::event_loop::ActiveEventLoop,
        _window: glium::winit::window::WindowId,
        event: glium::winit::event::WindowEvent,
    ) {
        // Look for changes in files
        if let Ok(_) = self.input_file_events.try_recv() {
            // TODO: best way to print errors?
            let program = Self::create_program(&self.display, &self.input_file);
            // TODO: handle error
            //       maybe move to own method? that way we can keep this render method as clean as possible
            match program {
                Ok(program) => {
                    self.state.program = program;
                    self.window.request_redraw();
                    println!("Refreshed program");
                },
                Err(err) => {
                    self.last_error = err;
                    // TODO: actually use the saved value in a UI
                    eprintln!("[SEPL-ERROR] {}", self.last_error);
                },
            }
        }

        // TODO: present compilation errors somewhere
        //       egui? that opens up possibilities for user configured GUIs?

        match event {
            glium::winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            glium::winit::event::WindowEvent::Resized(new_size) => {
                self.display.resize(new_size.into());
            }
            glium::winit::event::WindowEvent::RedrawRequested => {
                let mut frame = self.display.draw();
                frame.draw(
                    &self.state.vertex_buffer,
                    &self.state.index_buffer,
                    &self.state.program,
                    &self.state.uniforms,
                    &DrawParameters::default(),
                ).expect("Could not draw frame");

                frame.finish().expect("Could not switch framebuffers");
            }
            // TODO: other events
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &glium::winit::event_loop::ActiveEventLoop) {
        self.window.request_redraw();
    }
}
