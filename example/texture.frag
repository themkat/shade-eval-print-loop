#version 330 core

out vec4 color;

uniform float screen_width;
uniform float screen_height;
uniform sampler2D mytex;

uniform float elapsed_time;

void main() {
  vec2 uv = gl_FragCoord.xy / vec2(screen_width, screen_height);
  uv.y = 1.0 - uv.y;
  uv.y = uv.y + 0.5 * cos(elapsed_time + uv.x);
  
  vec4 tex = texture(mytex, uv).rgba;
  
  color = tex;
}
