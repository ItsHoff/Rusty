#version 330

in vec3 position;
in vec3 normal;

out vec3 v_normal;

uniform mat4 camera;
uniform mat4 matrix;
uniform mat4 perspective;

void main() {
    v_normal = transpose(inverse(mat3(matrix))) * normal;
    gl_Position = perspective * camera * matrix * vec4(position, 1.0);
}
