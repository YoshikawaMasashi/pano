#version 300 es
#define PI 3.1415926535897932384626

precision highp float;

in vec2 fragment_position;
out vec4 color;

uniform sampler2D front;
uniform sampler2D back;
uniform sampler2D left;
uniform sampler2D right;
uniform sampler2D top;
uniform sampler2D bottom;

void main() {
    float azimuth = fragment_position.x * PI;
    float elevation = fragment_position.y * PI / 2.0;
    
    vec3 pt;
    pt.x = cos(elevation) * sin(azimuth);
    pt.y = sin(elevation);
    pt.z = cos(elevation) * cos(azimuth);
    
    if ((abs(pt.x) >= abs(pt.y)) && (abs(pt.x) >= abs(pt.z))) {{
        if (pt.x <= 0.0) {{
            color = texture(left, vec2(((-pt.z/pt.x)+1.0)/2.0,((-pt.y/pt.x)+1.0)/2.0));
        }} else {{
            color = texture(right, vec2(((-pt.z/pt.x)+1.0)/2.0,((pt.y/pt.x)+1.0)/2.0));
        }}
    }} else if (abs(pt.y) >= abs(pt.z)) {{
        if (pt.y <= 0.0) {{
            color = texture(top, vec2(((-pt.x/pt.y)+1.0)/2.0,((-pt.z/pt.y)+1.0)/2.0));
        }} else {{
            color = texture(bottom, vec2(((pt.x/pt.y)+1.0)/2.0,((-pt.z/pt.y)+1.0)/2.0));
        }}
    }} else {{
        if (pt.z <= 0.0) {{
            color = texture(back, vec2(((pt.x/pt.z)+1.0)/2.0,((-pt.y/pt.z)+1.0)/2.0));
        }} else {{
            color = texture(front, vec2(((pt.x/pt.z)+1.0)/2.0,((pt.y/pt.z)+1.0)/2.0));
        }}
    }}
}
