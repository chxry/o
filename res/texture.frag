#version 330 core
in vec2 v_uv;

uniform sampler2D tex;

out vec4 f_color;

void main() {
	f_color = texture(tex, v_uv);
}
