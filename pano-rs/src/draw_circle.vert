#version 300 es

        in vec2 position;
        out vec2 v_tex_coords;

        void main() {
            v_tex_coords = position;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    