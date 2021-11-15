#version 300 es
#define PI 3.1415926535897932384626

const vec2[9] POSITIONS = vec2[](
    vec2(-1.0, -1.0),
    vec2(-1.0, 0.0),
    vec2(-1.0, 1.0),
    vec2(0.0, -1.0),
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, -1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);
const int[24] INDICES = int[](
    0, 1, 3,
    3, 4, 1,
    1, 2, 4,
    4, 5, 2,
    3, 4, 6,
    6, 7, 4,
    4, 5, 7,
    7, 8, 5
);

mat3 rotation_x(float rot) {
    return mat3(
        vec3(1, 0.0, 0.0),
        vec3(0.0, cos(rot), -sin(rot)),
        vec3(0.0, sin(rot), cos(rot))
    );
}
mat3 rotation_y(float rot) {
    return mat3(
        vec3(cos(rot), 0.0, sin(rot)),
        vec3(0.0, 1.0, 0.0),
        vec3(-sin(rot), 0.0, cos(rot))
    );
}
mat3 rotation_z(float rot) {
    return mat3(
        vec3(cos(rot), -sin(rot), 0.0),
        vec3(sin(rot), cos(rot), 0.0),
        vec3(0.0, 0.0, 1.0)
    );
}

uniform vec3 start_position;
uniform vec3 end_position;

out vec2 brush_position;

void main(void) {
    float width = 0.02;
    vec3 x1 = start_position / length(start_position);
    vec3 x2 = end_position / length(end_position);

    float dist = acos(abs(dot(x1, x2))) / width;

    vec3 center_position = (x1 + x2) / 2.0;
    float y_rot = asin(center_position.x / length(center_position.xz));
    mat3 y_rot_mat = rotation_y(y_rot);

    center_position = y_rot_mat * center_position;
    x1 = y_rot_mat * x1;
    x2 = y_rot_mat * x2;

    float x_rot = asin(center_position.y);
    mat3 x_rot_mat = rotation_x(-x_rot);
    center_position = x_rot_mat * center_position;
    x1 = x_rot_mat * x1;
    x2 = x_rot_mat * x2;

    float z_rot = -sign(x1.x) * asin(x1.y / length(x1.xy));
    mat3 z_rot_mat = rotation_z(-z_rot);
    center_position = z_rot_mat * center_position;
    x1 = z_rot_mat * x1;
    x2 = z_rot_mat * x2;

    float theta = asin(x1.x);
    float theta_sign = sign(theta);
    theta = abs(theta) + width;
    x1 = vec3(theta_sign * sin(theta), 0.0, cos(theta));
    x2 = vec3(-theta_sign * sin(theta), 0.0, cos(theta));

    float end_ratio = POSITIONS[INDICES[gl_VertexID]].x * 0.5 + 0.5;
    float start_ratio = 1.0 - end_ratio;
    vec3 target_position = x1 * start_ratio + x2 * end_ratio;
    target_position = target_position * cos(width * POSITIONS[INDICES[gl_VertexID]].y) + vec3(0, 1, 0) * sin(width * POSITIONS[INDICES[gl_VertexID]].y);
    target_position = target_position * z_rot_mat * x_rot_mat * y_rot_mat;

    float elevation = asin(target_position.y);
    float azimuth = sign(target_position.x) * acos(target_position.z / length(target_position.xz));

    gl_Position = vec4(azimuth / PI, -elevation / PI * 2.0, 0.0, 1.0);
    brush_position = vec2((dist + 2.0) * (0.5 * POSITIONS[INDICES[gl_VertexID]].x + 0.5) - 1.0, POSITIONS[INDICES[gl_VertexID]].y);
}
