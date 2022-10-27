#version 450 core

layout (location = 3) uniform vec3 u_color;

out vec4 f_color;

void main() {
	f_color = vec4(u_color, 1.0);
}
