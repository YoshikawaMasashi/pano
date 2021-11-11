#version 300 es
#define PI 3.1415926535897932384626

const vec2[4] POSITIONS = vec2[](
    vec2(-1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, -1.0)
);
const int[6] INDICES = int[](
    0, 1, 2,
    2, 3, 0
);

out vec2 fragment_position;

uniform float canvas_height;
uniform float canvas_width;
uniform float fov;

void main(void) {
    float canvas_size = min(canvas_height - 10.0, canvas_width - 10.0);

    vec2 position = POSITIONS[INDICES[gl_VertexID]];
    gl_Position = vec4(position.x * (canvas_size / canvas_width), position.y * (canvas_size / canvas_height), 0.0, 1.0);
    fragment_position = vec2(position.x, position.y) * tan(fov / 2.0 / 180.0 * PI);
}
