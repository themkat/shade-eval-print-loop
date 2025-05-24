#version 330 core

uniform float screen_width;
uniform float screen_height;

uniform float elapsed_time;

out vec4 color;

vec3 palette(float t, vec3 a, vec3 b, vec3 c, vec3 d ) {
    return a + b*cos( 6.283185*(c*t+d) );
}

void main() {
  vec2 uv = 2.0 * gl_FragCoord.xy / vec2(screen_width, screen_height) - 1.0;
  
  float val = 0.6 * 0.5*cos(length(vec2(uv.x + elapsed_time, uv.y) + 128.0))+ 0.5*sin(length(vec2(uv.x, uv.y + elapsed_time + 128.0))) + 0.5*sin(length(uv + 32.0)) + 0.5*sin(length(uv + 64.0)) + 0.5*sin(length(uv - 32.0));
  val = abs(val);
  val *= 10.0;

  color = vec4(palette(val,
                       vec3(0.2, 0.0, 0.4),
                       vec3(0.5, 0.1, 0.3),
                       vec3(0.1, 0.1, 0.2),
                       vec3(0.3, 0.1, 0.4)), 1.0);
}
