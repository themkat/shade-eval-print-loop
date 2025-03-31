use std::collections::HashMap;

use glium::{backend::glutin::SimpleWindowBuilder, glutin::surface::WindowSurface, winit::{application::ApplicationHandler, event_loop::EventLoop, window::Window}, Display};
use nalgebra::Matrix4;

// Separate scheme and render into modules
mod scheme;

enum UniformValue {
    // TODO: document that values are cast and coerced.
    UniformInt(i32),
    UniformFloat(f32),
    UniformMatrix4(Matrix4<f32>)
}

struct State {
    // TODO: just have an input channel? That the NetworkedScheme process sends to?

    // TODO: just store a glium dynamic uniforms instead? Tight coupling, but less bullshit
    uniforms: HashMap<String, UniformValue>
}

// TODO: maybe have an accessor to ensure non mutable hashmap access?

// TODO: method for update state etc.
//       check_for_updates?
//       how should we get get values in the renderer?
//       slice of Uniforms?

// TODO: how do we ensure loose coupling? should state call the scheme methods? or is there an intermediary that would make the code easier to read?
//       using just State and some traits that the scheme structs implement? then we can easily replace the source of state updates if ever necessary


// TODO: just do render here? Simple anyway..
pub fn init() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let (window, display) = SimpleWindowBuilder::new()
        .with_inner_size(1280, 720)
        .with_title("Shade Eval Print Loop")
        .build(&event_loop);

    let mut app = SEPLApp {window, display};

    event_loop.run_app(&mut app).expect("Could not run app");
}

struct SEPLApp {
    display: Display<WindowSurface>,
    window: Window,
}

impl ApplicationHandler for SEPLApp {
    fn resumed(&mut self, _event_loop: &glium::winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &glium::winit::event_loop::ActiveEventLoop,
        _window: glium::winit::window::WindowId,
        event: glium::winit::event::WindowEvent,
    ) {
        // TODO: any of our own logic that should be done each frame. Listen to events etc.

        // TODO: should we handle elapsed time here? or in the scheme code?
        
        match event {
            glium::winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            glium::winit::event::WindowEvent::RedrawRequested => {

                // TODO: render something
                
            },
            // TODO: other events
            _ => {}
        }
    }
}
