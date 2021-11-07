#version 300 es
#define PI 3.1415926535897932384626

precision highp float;

in vec2 fragment_position;
out vec4 color;

uniform float scale;
uniform vec3 position;
uniform vec4 circle_color;

void main() {
  float azimuth = fragment_position.x * PI;
  float elevation = fragment_position.y * PI / 2.0;

  vec3 pt;
  pt.x = cos(elevation) * sin(azimuth);
  pt.y = sin(elevation);
  pt.z = cos(elevation) * cos(azimuth);

  vec3 rotation_eular = -position / 180.0 * PI;
  mat3 rotation_x = mat3(
    vec3(1, 0.0, 0.0),
    vec3(0.0, cos(rotation_eular.x), -sin(rotation_eular.x)),
    vec3(0.0, sin(rotation_eular.x), cos(rotation_eular.x))
  );
  mat3 rotation_y = mat3(
    vec3(cos(rotation_eular.y), 0.0, sin(rotation_eular.y)),
    vec3(0.0, 1.0, 0.0),
    vec3(-sin(rotation_eular.y), 0.0, cos(rotation_eular.y))
  );
  mat3 rotation_z = mat3(
    vec3(cos(rotation_eular.z), -sin(rotation_eular.z), 0.0),
    vec3(sin(rotation_eular.z), cos(rotation_eular.z), 0.0),
    vec3(0.0, 0.0, 1.0)
  );
  mat3 rotation = rotation_x * rotation_y * rotation_z;
  pt = rotation * pt;

  if (pt.z >= 0.0) {
    vec2 plane_pos = vec2(pt.x / pt.z, pt.y / pt.z);
    if (sqrt(plane_pos.x * plane_pos.x + plane_pos.y * plane_pos.y) <= scale) {
      color = circle_color;
    } else {
      color = vec4(0.0, 0.0, 0.0, 0.0);
    }
  } else {
    color = vec4(0.0, 0.0, 0.0, 0.0);
  }
}
