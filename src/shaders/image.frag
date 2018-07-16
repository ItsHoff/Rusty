#version 330

in vec2 v_tex_coords;

out vec4 color;

uniform sampler2D image;

float hable(float x) {
    float A = 0.15;
    float B = 0.50;
    float C = 0.10;
    float D = 0.20;
    float E = 0.02;
    float F = 0.30;

    return ((x*(A*x+C*B)+D*E)/(x*(A*x+B)+D*F))-E/F;
}

void main() {
  color = texture(image, v_tex_coords);
  float luma = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
  // The last division defines the white point
  color.rgb *= hable(luma) / luma / hable(4000.0);
}
