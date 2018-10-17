#version 330

in vec3 pos;
in vec3 normal;
in vec2 tex_coords;

out vec3 v_normal;
out vec2 v_tex_coords;

uniform mat4 world_to_clip;

void main() {
    v_normal = normal;
    v_tex_coords = tex_coords;
    gl_Position = world_to_clip * vec4(pos, 1.0);
}
