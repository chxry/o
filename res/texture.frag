#version 450 core
in vec2 v_uv;

uniform sampler2D u_texture;

out vec4 f_color;

void main() {
	f_color = texture(u_texture, v_uv);
}
