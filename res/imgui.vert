 #version 450 core
layout (location = 0) in vec2 v_pos;
layout (location = 1) in vec2 v_uv;
layout (location = 2) in vec4 v_color;

layout (location = 0) uniform mat4 u_transform;

layout (location = 0) out vec2 a_uv;
layout (location = 1) out vec4 a_color;

void main() {
	a_uv = v_uv;
    a_color = v_color;
    gl_Position = u_transform * vec4(v_pos, 0.0, 1.0);
}