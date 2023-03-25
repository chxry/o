#version 330 core
in vec3 v_pos;
in vec3 v_normal;

uniform vec3 color;

out vec4 f_color;

#include "lighting.glsl"

void main() {
	f_color = vec4(lighting() * color, 1.0);
}
