#version 330 core
layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 uv;

uniform mat4 view;
uniform mat4 projection;

out vec3 v_pos;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_pos = transpose(mat3(view)) * (inverse(projection) * gl_Position).xyz;
}
