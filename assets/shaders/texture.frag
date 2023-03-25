#version 330 core
in vec3 v_pos;
in vec2 v_uv;
in vec3 v_normal;

uniform sampler2D tex;

out vec4 f_color;

#include "lighting.glsl"

void main() {
	vec3 color = texture(tex, v_uv).rgb;
	f_color = vec4(lighting() * color, 1.0);
}
