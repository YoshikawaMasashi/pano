#version 300 es
#define PI 3.1415926535897932384626

precision highp float;

in vec2 fragment_position;
out vec4 color;

uniform sampler2D tex;
uniform float rotation_x;
uniform float rotation_y;

void main(void) {
    float azimuth = fragment_position.x * PI;
    float elevation = fragment_position.y * PI / 2.0;

    vec3 pt;
    pt.x = cos(elevation) * sin(azimuth);
    pt.y = sin(elevation);
    pt.z = cos(elevation) * cos(azimuth);
    
    float rotation_x_ = rotation_x / 180.0 * PI;
    float rotation_y_ = rotation_y / 180.0 * PI;
    mat3 rotation_x_mat = mat3(
        vec3(1, 0.0, 0.0),
        vec3(0.0, cos(rotation_x_), -sin(rotation_x_)),
        vec3(0.0, sin(rotation_x_), cos(rotation_x_))
    );
    mat3 rotation_y_mat = mat3(
        vec3(cos(rotation_y_), 0.0, sin(rotation_y_)),
        vec3(0.0, 1.0, 0.0),
        vec3(-sin(rotation_y_), 0.0, cos(rotation_y_))
    );
    mat3 rotation = rotation_y_mat * rotation_x_mat;
    pt = pt * rotation;

    elevation = asin(pt.y);
    azimuth = sign(pt.x) * acos(pt.z / length(pt.xz)); // sign(pt.x) * acos(pt.z / cos(elevation));

    vec2 tex_coords = vec2(azimuth / PI, elevation / PI * 2.0);
    tex_coords = (tex_coords + 1.0) / 2.0;

    color = texture(tex, tex_coords);
}
