use glium::{
    Blend, DrawParameters, Program, Surface, Texture2d, VertexBuffer, backend::Facade,
    index::NoIndices, uniform, uniforms::EmptyUniforms,
};
use rusttype::{Font, Scale, point};

use crate::geometry::{SQUARE, Vertex};

const TEXT_RENDER_VERTEX_SHADER: &str = include_str!("../shaders/pass_text.vert");

const TEXT_RENDER_FRAGMENT_SHADER: &str = include_str!("../shaders/text.frag");

const SOLID_PLANE_VERTEX_SHADER: &str = include_str!("../shaders/pass.vert");

const SOLID_PLANE_FRAGMENT_SHADER: &str = include_str!("../shaders/solid.frag");

pub struct TextRenderer {
    font: Font<'static>,
    program: Program,
    solid_background_program: Program,
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
        let solid_background_program = Program::from_source(
            display,
            SOLID_PLANE_VERTEX_SHADER,
            SOLID_PLANE_FRAGMENT_SHADER,
            None,
        )
        .expect("Could not create program for solid square of fixed color.");

        let vertex_buffer = VertexBuffer::new(display, &SQUARE).unwrap();
        let index_buffer = NoIndices(glium::index::PrimitiveType::TriangleStrip);

        Self {
            font: Font::try_from_bytes(include_bytes!("../fonts/OpenSans-Bold.ttf")).unwrap(),
            program,
            solid_background_program,
            vertex_buffer,
            index_buffer,
            prev_text: None,
            prev_texture: None,
        }
    }

    fn render_text_to_texture<T: Facade>(&self, display: &T, text: &str) -> Texture2d {
        let mut result = vec![vec![(0, 0, 0, 0); 800]; 500];

        // TODO: maybe prettier formatting?
        // only take the last first error line. The rest are usually caused by this first one
        let text = wrap_words(text.lines().nth(0).unwrap_or("").to_string(), 40);
        let mut y_pos = 48.0;
        for line in text.lines() {
            let scale = Scale::uniform(48.0);
            let glyphs = self.font.layout(line, scale, point(20.0, y_pos));

            // TODO: use glyphs to decide on a suitable size
            for glyph in glyphs {
                // only rasterize text that have data
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    glyph.draw(|x, y, v| {
                        result[(bounding_box.min.y as u32 + y) as usize]
                            [(bounding_box.min.x as u32 + x) as usize] =
                            (255, 0, 0, (v * 255.0) as u8);
                    });
                }
            }

            // tweaked number for spacing between lines
            y_pos += 32.0;
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

        // solid plane to make text pop out more if the user have an active fragment shader
        surface
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.solid_background_program,
                &EmptyUniforms,
                &DrawParameters {
                    blend: Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .expect("Could not draw solid plane");

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

/// Takes a maximum length as an input and wraps the words by putting as many words on a line as possible.
fn wrap_words(text: String, line_length: usize) -> String {
    let mut leftover = line_length;
    let mut result = String::with_capacity(text.len());

    for word in text.split_whitespace() {
        if (word.len() + 1) > leftover {
            result += "\n";
            leftover = line_length;
        }
        result += word;
        result += " ";
        leftover = leftover.saturating_sub(word.len() + 1);
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::text::wrap_words;

    #[test]
    fn wrap_words_test() {
        assert_eq!(
            "Hi \nthere ".to_string(),
            wrap_words("Hi there".to_string(), 4)
        );
        assert_eq!("We gotta burn the \nrain forest, dump \ntoxic waste, \npollute the air, \nand rip up the \nOZONE! ".to_string(), wrap_words("We gotta burn the rain forest, dump toxic waste, pollute the air, and rip up the OZONE!".to_string(), 20));
    }
}
