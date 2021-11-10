#version 300 es
#define PI 3.1415926535897932384626
#define X 0.525731112119133606
#define Z 0.850650808352039932
#define N 0.0

precision highp float;

in vec2 fragment_position;
out vec4 color;

uniform sampler2D tex;

const vec3[12] ICO_VERTICES = vec3[](
    vec3(-X, N, Z),
    vec3(X, N, Z),
    vec3(-X, N, -Z),
    vec3(X, N, -Z),
    vec3(N, Z, X),
    vec3(N, Z, -X),
    vec3(N, -Z, X),
    vec3(N, -Z, -X),
    vec3(Z, X, N),
    vec3(-Z, X, N),
    vec3(Z, -X, N),
    vec3(-Z, -X, N)
);
const int[60] ICO_INDICES = int[](
    0, 4, 1,
    0, 9, 4,
    9, 5, 4,
    4, 5, 8,
    4, 8, 1,
    8, 10, 1,
    8, 3, 10,
    5, 3, 8,
    5, 2, 3,
    2, 7, 3,
    7, 10, 3,
    7, 6, 10,
    7, 11, 6,
    11, 0, 6,
    0, 1, 6,
    6, 1, 10,
    9, 0, 11,
    9, 11, 2,
    9, 2, 5,
    7, 2, 11
);

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

void main() {
    int res = 180;

    color = vec4(0.0, 0.0, 0.0, 0.0);

    for (int triangle = 0; triangle < 20; triangle++){
        for(int i = 0; i <= res; i++) {
            for(int j = 0; j <= res - i; j++) {
                int k = res - i - j;
                vec3 ratio = vec3(float(i) / float(res), float(j) / float(res), float(k) / float(res));
                vec3 v1 = vec3(ICO_VERTICES[ICO_INDICES[triangle * 3]]);
                vec3 v2 = vec3(ICO_VERTICES[ICO_INDICES[triangle * 3 + 1]]);
                vec3 v3 = vec3(ICO_VERTICES[ICO_INDICES[triangle * 3 + 2]]);
                vec3 v = v1 * ratio.x + v2 * ratio.y + v3 * ratio.z;
                v.x += 0.003 * (2.0 * rand(v.yz) - 1.0);
                v.y += 0.003 * (2.0 * rand(v.zx) - 1.0);
                v.z += 0.003 * (2.0 * rand(v.xy) - 1.0);
                v = normalize(v);
                
                float v_elevation = asin(v.y);
                float v_azimuth = sign(v.x) * acos(v.z / cos(v_elevation));
                vec4 v_color = texture(tex, vec2(0.5 * v_azimuth / PI + 0.5, 0.5 * v_elevation / PI * 2.0 + 0.5));

                vec3 position = vec3(-v_elevation * 180.0 / PI, -v_azimuth * 180.0 / PI, 0.0);
                // TODO: Color Random

                float scale = 0.005 + 0.005 * rand(vec2(v_azimuth + float(i), v_elevation + float(j)));
                
                float azimuth = fragment_position.x * PI;
                float elevation = -fragment_position.y * PI / 2.0;

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
                        color = v_color;
                    }
                }
            }
        }
    }
}
