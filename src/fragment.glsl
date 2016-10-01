#version 330

in vec3 v_normal;

out vec4 color;

uniform vec3 u_light;
uniform vec3 u_color;

void main() {
  float brightness = dot(normalize(v_normal), normalize(u_light));
  vec3 dark_color = 0.5 * u_color;
  vec3 regular_color = u_color;
  color = vec4(mix(dark_color, regular_color, brightness), 1.0);
}
