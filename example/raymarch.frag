#version 330 core

uniform float screen_width;
uniform float screen_height;

uniform float elapsed_time;

const int MAX_ITERATIONS = 1000;
const float MAX_DISTANCE = 10000.0;

out vec4 color;

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

// standard GLSL noise function of internet legend. Adapted to 3 dims
float noise(vec3 co){
  return fract(sin(dot(co, vec3(12.9898, 78.233, 69.78))) * 43758.5453);
}

// iqquillez fbm subtraction logic
// renamed for clarity. Found no better way of doing it, so copy paste it is :P
float random_radius_sphere( ivec3 i, vec3 f, ivec3 c ) {
  float rad = 0.5*noise((i + c));
  return length(f-vec3(c)) - rad;
}

float eight_random_spheres(vec3 p) {
  ivec3 i = ivec3(floor(p));
  vec3 f = fract(p);
  return min(min(min(random_radius_sphere(i,f,ivec3(0,0,0)),
                     random_radius_sphere(i,f,ivec3(0,0,1))),
                 min(random_radius_sphere(i,f,ivec3(0,1,0)),
                     random_radius_sphere(i,f,ivec3(0,1,1)))),
             min(min(random_radius_sphere(i,f,ivec3(1,0,0)),
                     random_radius_sphere(i,f,ivec3(1,0,1))),
                 min(random_radius_sphere(i,f,ivec3(1,1,0)),
                     random_radius_sphere(i,f,ivec3(1,1,1)))));
}

// </iqquillez fbm subtraction logic>

// box, again thanks to iqquillez
float round_box(vec3 p, vec3 b, float r) {
  vec3 q = abs(p) - b + r;
  return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0) - r;
}

Hit blob_structure(vec3 pos) {
  int num_spheres = 8;
  float dist = sphere(pos, vec3(0.0), 1.0);
  for(int i = 1; i < num_spheres; i++) {
    dist = smin(dist, sphere(pos, vec3(cos(elapsed_time / i), -tan(elapsed_time/i), sin(elapsed_time)), 0.7), 0.2);
  }
  return Hit(smin(dist, round_box(pos + vec3(0.0, 4.0, 0.0), vec3(3.0, 1.0, 3.0), 0.5), 0.2), 0);
}

float eroded_block(vec3 pos, vec3 dims) {
  float dist = round_box(pos, dims, 0.2);
  // fbm from iq
  float s = 1.0;
  for( int i=0; i<4; i++ ) {
    float n = s*eight_random_spheres(pos);
    dist = max( dist, -n);
    pos = mat3(0.00, 1.60, 1.20, 
               -1.60, 0.72,-0.96,
               -1.20,-0.96, 1.28)*pos;
    s = 0.5*s;
  }

  return dist;
}

// a weird basin containing the pink blobby goo
Hit basin(vec3 pos) {
  float dist = eroded_block(pos + vec3(4.0, 5.0, 0.0), vec3(2.0, 4.0, 4.0));
  dist = min(dist, eroded_block(pos + vec3(-4.0, 5.0, 0.0), vec3(2.0, 4.0, 4.0)));

  float hole_box = eroded_block(pos + vec3(0.0, 4.0, 4.0), vec3(3.0, 10.0, 0.5));
  float smaller_box = round_box(pos + vec3(0.0, -4.0, 4.0), vec3(2.0, 1.0, 0.8), 0.2);
  dist = min(dist, max(hole_box, -smaller_box));
  
  return Hit(dist, 1);
}

Hit scene(vec3 pos) {
  return hit_min(hit_min(blob_structure(pos + vec3(0.0, 0.0, 2.0)), basin(pos + vec3(0.0, 0.0, 2.0))),
                 Hit(plane(pos, vec3(0.0, 1.0, 0.0), 4.0), 2));
}

vec3 normal(vec3 pos) {
  return normalize(vec3(scene(pos + vec3(0.01, 0.0, 0.0)).dist - scene(pos - vec3(0.01, 0.0, 0.0)).dist,
                        scene(pos + vec3(0.0, 0.01, 0.0)).dist - scene(pos - vec3(0.0, 0.01, 0.0)).dist,
                        scene(pos + vec3(0.0, 0.0, 0.01)).dist - scene(pos - vec3(0.0, 0.0, 0.01)).dist));
}

// probably unintuitive method as well. TOok it out of my ass
float shadow_march(vec3 pos, vec3 ray_dir) {
  float t = 0.5;
  while(t < 200.0) {
    float dist = scene(pos + t*ray_dir).dist;
    if(dist < 0.001) {
      // could be 0, but like the way occluding factor 0.1 looks like
      return 0.1;
    }
    
    t+= dist;
  }

  return 1.0;
}

void main() {
  // upside down, but saves a calc
  vec2 uv = gl_FragCoord.xy / vec2(screen_width, screen_height);
  vec3 ray_dir = vec3(uv * 2.0 - 1.0, -1.0);
  vec3 cam_pos = vec3(0.0, 0.0, 5.0);

  vec3 light_pos = vec3(3.0, 6.0, 5.0);
  
  float t = 0.0;
  int i = 0;
  while (i < MAX_ITERATIONS && t < MAX_DISTANCE) {
    vec3 pos = cam_pos + t*ray_dir;
    Hit hit = scene(pos);
    if (hit.dist < 0.001) {
      vec3 light_dir = normalize(light_pos - pos);
      vec3 normal = normal(pos);
      float diffuse_intensity = max(0.0, dot(light_dir, normal));
      vec3 half_vec = normalize(light_dir + -ray_dir);
      float specular = max(0.0, pow(dot(half_vec, normal), 256));
      float shadow = shadow_march(pos, light_dir);
      
      // stupid material handling
      vec3 ambient_color = vec3(1.0, 0.8, 0.9);
      vec3 diffuse_color = vec3(1.0, 0.8, 0.9);
      if (hit.material_id == 1) {
        ambient_color = vec3(0.1, 0.1, 0.1);
        diffuse_color = vec3(0.4, 0.4, 0.4);
        specular = 0.0;
      } else if (hit.material_id == 2) {
        ambient_color = vec3(0.7, 0.9, 0.7);
        // TODO: more interesting material
        diffuse_color = noise(pos) * vec3(0.9, 0.9, 0.9);
        specular = 0.0;
      }
      
      color = vec4(0.3*ambient_color + shadow*diffuse_color * diffuse_intensity + shadow*vec3(1.0)*specular, 1.0);
      return;
    }

    t += hit.dist;
  }

  // TODO: more interesting color
  color = mix(vec4(0.9, 0.9, 0.9, 1.0), vec4(0.4, 0.6, 0.8, 1.0), 0.8 - uv.y / 2.0);
}
