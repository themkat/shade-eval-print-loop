use glium::{
    Blend, DrawParameters, Program, Surface, Texture2d, VertexBuffer, backend::Facade,
    index::NoIndices, uniform,
};
use rusttype::{Font, Scale, point};

use crate::geometry::{SQUARE, Vertex};

const TEXT_RENDER_VERTEX_SHADER: &str = include_str!("../shaders/pass_text.vert");

const TEXT_RENDER_FRAGMENT_SHADER: &str = include_str!("../shaders/text.frag");

pub struct TextRenderer {
    font: Font<'static>,
    // TODO: consider re-using the GLState struct
    program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: NoIndices,

    // some utility caching to avoid recreating same text twice
    prev_text: Option<String>,
    prev_texture: Option<Texture2d>,
}

impl TextRenderer {
    pub fn new<T: Facade>(display: &T) -> Self {
        let program = Program::from_source(
            display,
            TEXT_RENDER_VERTEX_SHADER,
            TEXT_RENDER_FRAGMENT_SHADER,
            None,
        )
        .unwrap();
        let vertex_buffer = VertexBuffer::new(display, &SQUARE).unwrap();
        let index_buffer = NoIndices(glium::index::PrimitiveType::TriangleStrip);

        Self {
            font: Font::try_from_bytes(include_bytes!("../fonts/OpenSans-Bold.ttf")).unwrap(),
            program,
            vertex_buffer,
            index_buffer,
            prev_text: None,
            prev_texture: None,
        }
    }

    fn render_text_to_texture<T: Facade>(&self, display: &T, text: &str) -> Texture2d {
        let scale = Scale::uniform(32.0);
        let glyphs = self.font.layout(text, scale, point(20.0, 20.0));

        // TODO: process text. so we avoid too long text for single lines.

        // TODO: use glyphs to decide on a suitable size
        let mut result = vec![vec![(0, 0, 0, 0); 800]; 500];
        for glyph in glyphs {
            // only rasterize text that have data
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    result[(bounding_box.min.y as u32 + y) as usize]
                        [(bounding_box.min.x as u32 + x) as usize] = (255, 0, 0, (v * 255.0) as u8);
                });
            }
        }

        Texture2d::new(display, result).unwrap()
    }

    /// Render text to the specified surface.
    pub fn render_text<T: Facade, S: Surface>(&mut self, display: &T, surface: &mut S, text: &str) {
        if Some(text.to_string()) != self.prev_text {
            let texture = self.render_text_to_texture(display, text);
            self.prev_texture = Some(texture);
            self.prev_text = Some(text.to_string());
        }

        // we know at this point that there should always be a texture present
        let texture = self.prev_texture.as_ref().unwrap();

        surface.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniform! {
            font_texture: texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear).minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
        }, &DrawParameters {
            blend: Blend::alpha_blending(),
            ..Default::default()
        }).expect("Could not draw text to screen");
    }
}
