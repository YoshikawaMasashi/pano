#version 300 es
#define PI 3.1415926535897932384626

precision highp float;

out vec4 color;

in vec2 brush_position;

uniform int point_num;
uniform float point_offset;

void main(void) {
    color = vec4(1.0, 0.0, 0.0, 0.0);

    for(int i = 0; i < point_num; i++) {
        if (length(vec2(brush_position.x - (1.0 - point_offset) - float(i), brush_position.y)) < 0.5 ) {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    }
}
