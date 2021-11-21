#version 300 es
// https://www.shadertoy.com/view/MtBGRD
#define PI 3.1415926535897932384626
precision highp float;
in vec2 fragment_position;
out vec4 color;

uniform sampler2D tex;

mat3 cube_face_permutation_matrix(in vec3 p) {
    vec3 a = abs(p);
    float c = max(max(a.x, a.y), a.z);    

    vec3 s = c == a.x ? vec3(1.,0,0) : c == a.y ? vec3(0,1.,0) : vec3(0,0,1.);

    s *= sign(dot(p, s));
    vec3 q = s.yzx;
    return mat3(cross(q,s), q, s);
}

float face_id(vec3 axis) {
    float idx = dot(abs(axis), vec3(0.0, 2.0, 4.0));
    if (dot(axis, vec3(1.0)) < 0.0) { idx += 1.0; }
    
    return idx;
}

#define HASHSCALE3 vec3(.1031, .1030, .0973)

vec3 hash33(vec3 p3) {
	p3 = fract(p3 * HASHSCALE3);
    p3 += dot(p3, p3.yxz+19.19);
    return fract((p3.xxy + p3.yxx)*p3.zyx);

}

#define MAGIC_ANGLE 0.868734829276 // radians

float warp_theta = MAGIC_ANGLE;
float tan_warp_theta = tan(MAGIC_ANGLE);

/* Warp to go cube -> sphere */
vec2 warp(vec2 x) {
    return tan(warp_theta*x)/tan_warp_theta;
}

/* Unwarp to go sphere -> cube */
vec2 unwarp(vec2 x) {
    return atan(x*tan_warp_theta)/warp_theta; 
}

void main() {
    color = vec4(0.0, 0.0, 0.0, 0.0);
    
    float azimuth = fragment_position.x * PI;
    float elevation = fragment_position.y * PI / 2.0;
    float N = 128.0;
    float scale_constant = 0.005;

    vec3 pt;
    pt.x = cos(elevation) * sin(azimuth);
    pt.y = sin(elevation);
    pt.z = cos(elevation) * cos(azimuth);

    mat3 PT = cube_face_permutation_matrix(pt);

    vec3 cf = pt * PT; 
    cf /= cf.z;
    vec2 uv = unwarp(cf.xy);

    vec2 uv_quantized = floor(0.5 * N * uv + 0.5) * 2.0 / N;
    vec3 cf_quantized = vec3(uv_quantized, 1.0);
    for (int iter_idx = 0; iter_idx < 10; iter_idx++ ) {
        vec3 hash_quantized = hash33(vec3(face_id(PT[2]), uv_quantized) * 351.09782 + float(iter_idx) * vec3(9.134, 5.45, 34.9537));

        float current_maximum_z_idx = hash_quantized.z - 1.0;
        for (int du = -1; du <= 1; du++) {
            for (int dv = -1; dv <= 1; dv++) {
                vec2 uv_neighbor = cf_quantized.xy + vec2(float(du), float(dv)) * 2.0 / N;
                vec2 extra = abs(uv_neighbor - clamp(uv_neighbor, -1.0, 1.0));
                if (min(extra.x, extra.y) > 0.0) {
                    continue;
                }

                vec3 cf_neighbor = vec3(uv_neighbor, 1.0);
                vec3 pt_neighbor = PT * cf_neighbor;
                mat3 PT_neighbor = cube_face_permutation_matrix(pt_neighbor);

                cf_neighbor = pt_neighbor * PT_neighbor;
                cf_neighbor /= cf_neighbor.z;
                uv_neighbor = floor(0.5 * N * cf_neighbor.xy + 0.5) * 2.0 / N;
                vec3 hash_neighbor = hash33(vec3(face_id(PT_neighbor[2]), uv_neighbor) * 351.09782 + float(iter_idx) * vec3(9.134, 5.45, 34.9537));
                uv_neighbor += (hash_neighbor.xy - 0.5) * 2.0 / N;
                uv_neighbor = warp(uv_neighbor);
                cf_neighbor = vec3(uv_neighbor, 1.0);
                pt_neighbor = PT_neighbor * cf_neighbor / length(cf_neighbor);

                float dist = acos(dot(pt, pt_neighbor) / length(pt) / length(pt_neighbor));

                float scale = scale_constant + (scale_constant / 2.0) * (hash_neighbor.x + hash_neighbor.y + hash_neighbor.z) / 3.0;

                if(dist < scale) {
                    float current_z_idx = hash_neighbor.z;

                    if (current_z_idx > current_maximum_z_idx) {
                        float neighbor_elevation = asin(clamp(pt_neighbor.y, -1.0 + 1e-10, 1.0 - 1e-10));
                        float neighbor_azimuth = sign(pt_neighbor.x) * acos(clamp(pt_neighbor.z / cos(neighbor_elevation), -1.0 + 1e-10, 1.0 - 1e-10));
                        vec2 neighbor_position = vec2(0.5 * neighbor_azimuth / PI + 0.5, 0.5 * neighbor_elevation / PI * 2.0 + 0.5);
                        vec4 neighbor_color = texture(tex, neighbor_position);
                        color = neighbor_color;
        
                        current_maximum_z_idx = current_z_idx;
                    }
                }
            }
        }
    }
}
