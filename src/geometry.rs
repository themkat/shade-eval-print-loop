use glium::implement_vertex;

#[derive(Clone, Copy)]
pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    const fn new(x: f32, y: f32) -> Self {
        Self { position: [x, y] }
    }
}

implement_vertex!(Vertex, position);

// Just a quad that covers the screen
pub const SQUARE: [Vertex; 4] = [
    Vertex::new(-1.0, 1.0),
    Vertex::new(-1.0, -1.0),
    Vertex::new(1.0, 1.0),
    Vertex::new(1.0, -1.0),
];
