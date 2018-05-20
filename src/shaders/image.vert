#version 330

in vec3 pos;
in vec3 normal;
in vec2 tex_coords;

out vec2 v_tex_coords;

void main() {
    v_tex_coords = tex_coords;
    gl_Position = vec4(pos, 1.0);
}
