#version 330 core

uniform float screen_width;
uniform float screen_height;

uniform float elapsed_time;

const int MAX_ITERATIONS = 1000;
const float MAX_DISTANCE = 10000.0;

out vec4 color;

// TODO: a struct of dist + material id? 
struct Hit {
  float dist;
  int material_id;
};

Hit hit_min(Hit hit1, Hit hit2) {
  if (hit1.dist < hit2.dist) {
    return hit1;
  }

  return hit2;
}

float plane(vec3 pos, vec3 normal, float height) {
  return dot(pos, normal) + height;
}

float sphere(vec3 pos, vec3 center, float radius) {
  return length(pos - center) - radius;
}

// smoothed minimum function using exponentials
// thanks iq: https://iquilezles.org/articles/smin/
float smin( float a, float b, float k ) {
    k *= 1.0;
    float r = exp2(-a/k) + exp2(-b/k);
    return -k*log2(r);
}

// box, again thanks to iquilezlez
float round_box(vec3 p, vec3 b, float r) {
  vec3 q = abs(p) - b + r;
  return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0) - r;
}


// TODO: maybe have a function for our blob structure here? then the scene can just call that function and have botht he distance to it + material?
//       then we can have some blocks or other stone material using some noise to create them

float scene(vec3 pos) {
  int num_spheres = 10;
  float dist = sphere(pos, vec3(0.0), 1.0);
  for(int i = 1; i < num_spheres; i++) {
    dist = smin(dist, sphere(pos, vec3(cos(elapsed_time / i), -tan(elapsed_time/i), sin(elapsed_time)), 0.7), 0.2);
  }
  return smin(dist, plane(pos, vec3(0.0, 1.0, 0.0), 4.0), 0.2);
}

vec3 normal(vec3 pos) {
  return normalize(vec3(scene(pos + vec3(0.01, 0.0, 0.0)) - scene(pos - vec3(0.01, 0.0, 0.0)),
                        scene(pos + vec3(0.0, 0.01, 0.0)) - scene(pos - vec3(0.0, 0.01, 0.0)),
                        scene(pos + vec3(0.0, 0.0, 0.01)) - scene(pos - vec3(0.0, 0.0, 0.01))));
}

// TODO: refactor to a general raymarching
// probably unintuitive method as well. TOok it out of my ass
float shadow_march(vec3 pos, vec3 ray_dir) {
  float t = 0.5;
  while(t < 200.0) {
    float dist = scene(pos + t*ray_dir);
    if(dist < 0.001) {
      // could be 0, but like the way occluding factor 0.1 looks like
      return 0.1;
    }
    
    t+= dist;
  }

  return 1.0;
}

void main() {
  // upside down?
  vec2 uv = gl_FragCoord.xy / vec2(screen_width, screen_height);
  vec3 ray_dir = vec3(uv * 2.0 - 1.0, -1.0);
  vec3 cam_pos = vec3(0.0, 0.0, 5.0);

  vec3 light_pos = vec3(3.0, 6.0, 5.0);
  
  float t = 0.0;
  int i = 0;
  while (i < MAX_ITERATIONS && t < MAX_DISTANCE) {
    vec3 pos = cam_pos + t*ray_dir;
    float dist = scene(pos);
    // TODO: more advanced color
    if (dist < 0.001) {
      vec3 light_dir = normalize(light_pos - pos);
      vec3 normal = normal(pos);
      float diffuse_intensity = max(0.0, dot(light_dir, normal));
      vec3 half_vec = normalize(light_dir + -ray_dir);
      float specular = max(0.0, pow(dot(half_vec, normal), 256));
      float shadow = shadow_march(pos, light_dir);
      
      // TODO: what is the best way to have different colors for each material?
      
      
      color = vec4(0.3*vec3(1.0, 0.8, 0.9) + shadow*vec3(1.0, 0.8, 0.9) * diffuse_intensity + shadow*vec3(1.0)*specular, 1.0);
      return;
    }

    t += dist; 
  }

  color = vec4(0.0);
}
