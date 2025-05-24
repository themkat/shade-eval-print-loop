#version 330 core

uniform float screen_width;
uniform float screen_height;

uniform float elapsed_time;

out vec4 color;

// heart distance function from iqquillez
float sdHeart(vec2 p) {
    p.x = abs(p.x);

    if( p.y+p.x>1.0 )
      return sqrt(dot(p-vec2(0.25,0.75), p-vec2(0.25,0.75))) - sqrt(2.0)/4.0;
    return sqrt(min(dot(p-vec2(0.00,1.00), p-vec2(0.00,1.00)),
                    dot(p-0.5*max(p.x+p.y,0.0), p-0.5*max(p.x+p.y,0.0)))) * sign(p.x-p.y);
}

vec3 palette(float t, vec3 a, vec3 b, vec3 c, vec3 d ) {
    return a + b*cos( 6.283185*(c*t+d) );
}

// infamous "noise" / random oneliner
float noise(vec2 co) {
  return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

void main() {
  vec2 uv = 2.0 * gl_FragCoord.xy / vec2(screen_width, screen_height) - 1.0;
  uv *= 2.0;

  vec2 uv_sub = fract(uv);
  uv_sub = uv_sub * 2.0 - 1.0;
  uv_sub *= 2.0;
  uv_sub.y += 0.5;
    
  float dist = sdHeart(uv_sub) + cos(elapsed_time + noise(floor(uv)));
  
  //dist -= 0.5;
  dist = abs(dist);
  dist = smoothstep(0.1, 0.5, dist);

  color = vec4(palette(dist,
                       vec3(0.9, 0.6, 0.7),
                       vec3(0.5, 0.5, 0.5),
                       vec3(0.6, 0.6, 0.6),
                       vec3(1.0, 0.0, 1.0)), 1.0);
}

