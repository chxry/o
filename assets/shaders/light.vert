#version 330 core
layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec2 v_uv;

out vec2 uv;

void main() {
  uv = v_uv;
  gl_Position = vec4(v_pos, 1.0);
}