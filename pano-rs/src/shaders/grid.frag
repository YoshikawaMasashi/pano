#version 300 es
#define PI 3.1415926535897932384626

precision highp float;

in vec2 fragment_position;
out vec4 color;

uniform float rotation_x;
uniform float rotation_y;

void main(void) {
    vec3 pt = vec3(fragment_position.x, -fragment_position.y, 1.0);
    pt = normalize(pt);

    float value = 0.0;
    
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
    pt = rotation * pt;

    for (int x = -10; x <= 10; x+=5) {
        if (abs(pt.x) > 0.1) {
            vec3 intersection = pt / pt.x * float(x);
            if (length(intersection) > 0.3) {
                {
                    float buf = abs(intersection.y - 1.0 * round(intersection.y / 1.0));
                    value = max(value, exp(-5000.0 * buf * buf) * exp(-0.01 * length(intersection) * length(intersection)));
                }                
                {
                    float buf = abs(intersection.z - 1.0 * round(intersection.z / 1.0));
                    value = max(value, exp(-5000.0 * buf * buf) * exp(-0.01 * length(intersection) * length(intersection)));
                }
            }
        }
    }
    
    for (int y = -10; y <= 10; y+=5) {
        if (abs(pt.y) > 0.1) {
            vec3 intersection = pt / pt.y * float(y);
            if (length(intersection) > 0.3) {
                {
                    float buf = abs(intersection.z - 1.0 * round(intersection.z / 1.0));
                    value = max(value, exp(-5000.0 * buf * buf) * exp(-0.01 * length(intersection) * length(intersection)));
                }                
                {
                    float buf = abs(intersection.x - 1.0 * round(intersection.x / 1.0));
                    value = max(value, exp(-5000.0 * buf * buf) * exp(-0.01 * length(intersection) * length(intersection)));
                }
            }
        }
    }
    
    for (int z = -10; z <= 10; z+=5) {
        if (abs(pt.z) > 0.1) {
            vec3 intersection = pt / pt.z * float(z);
            if (length(intersection) > 0.3) {
                {
                    float buf = abs(intersection.x - 1.0 * round(intersection.x / 1.0));
                    value = max(value, exp(-5000.0 * buf * buf) * exp(-0.01 * length(intersection) * length(intersection)));
                }                
                {
                    float buf = abs(intersection.y - 1.0 * round(intersection.y / 1.0));
                    value = max(value, exp(-5000.0 * buf * buf) * exp(-0.01 * length(intersection) * length(intersection)));
                }
            }
        }
    }

    color = vec4(0.5, 0.5, 0.5, value);
}
