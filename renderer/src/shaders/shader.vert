layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coords;

out vec4 clip_position;
out vec2 frag_tex_coords;

void main() {
    frag_tex_coords = tex_coords;
    clip_position = vec4(position, 1.0);
}
