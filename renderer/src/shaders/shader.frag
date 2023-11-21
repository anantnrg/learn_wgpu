layout(binding = 0) uniform sampler2D t_diffuse;
layout(binding = 1) uniform sampler2D s_diffuse;

in vec2 frag_tex_coords;

out vec4 fragColor;

void main() {
    fragColor = texture(t_diffuse, frag_tex_coords);
}
