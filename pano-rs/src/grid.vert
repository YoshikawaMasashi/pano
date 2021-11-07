#version 300 es

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

void main(void) {
    vec2 position = POSITIONS[INDICES[gl_VertexID]];
    gl_Position = vec4(position, 0.0, 1.0);
    fragment_position = vec2(position.x, -position.y);
}
