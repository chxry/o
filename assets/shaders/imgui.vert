#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec4 color;

uniform mat4 transform;

out vec2 frag_uv;
out vec4 frag_color;

void main() {
	frag_uv = uv;
    frag_color = color;
    gl_Position = transform * vec4(pos, 0.0, 1.0);
}