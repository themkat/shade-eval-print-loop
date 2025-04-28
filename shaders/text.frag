#version 330 core

in vec2 uv;
out vec4 color;

uniform sampler2D font_texture;

void main() {
  vec4 font_color = texture(font_texture, uv);

  if (font_color.a <= 0.0) {
    discard;
  }

  // black border
  if (font_color.a <= 0.97) {
    color = vec4(0.0, 0.0, 0.0, 1.0);
  } else {
    color = font_color;
  }
}
