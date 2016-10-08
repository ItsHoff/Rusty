#version 330

in vec3 position;
in vec3 normal;
in vec2 tex_coords;

out vec3 v_normal;
out vec2 v_tex_coords;

uniform mat4 local_to_world;
uniform mat4 world_to_clip;

void main() {
    v_normal = transpose(inverse(mat3(local_to_world))) * normal;
    v_tex_coords = tex_coords;
    gl_Position = world_to_clip * local_to_world * vec4(position, 1.0);
}
