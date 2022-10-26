#version 450 core
in vec3 a_pos;
in vec2 a_uv;

layout (location = 0) uniform mat4 model;
layout (location = 1) uniform mat4 view;
layout (location = 2) uniform mat4 projection;

out vec2 v_uv;

void main() {
	v_uv = a_uv;
	gl_Position = projection * view * model * vec4(a_pos, 1.0);
}
