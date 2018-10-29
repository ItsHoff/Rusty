#version 330

in vec3 v_normal;
in vec2 v_tex_coords;

out vec4 color;

uniform vec3 u_light;
uniform vec3 u_color;
uniform bool u_has_texture;
uniform bool u_is_emissive;
uniform sampler2D tex;

void main() {
    float brightness = dot(normalize(v_normal), normalize(u_light));
    vec3 d_color;
    if (u_is_emissive) {
        d_color = vec3(255, 255, 0);
    } else if (u_has_texture) {
        d_color = vec3(texture(tex, v_tex_coords));
    } else {
        d_color = u_color;
    }
    vec3 dark_color = 0.5 * d_color;
    vec3 regular_color = d_color;
    color = vec4(mix(dark_color, regular_color, brightness), 1.0);
}
