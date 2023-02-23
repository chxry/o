#version 330 core
layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 v_pos;
out vec2 v_uv;
out vec3 v_normal;

void main() {
    v_pos = vec3(model * vec4(pos, 1.0));
    v_uv = uv;
    v_normal = mat3(transpose(inverse(model))) * normal;  
    gl_Position = projection * view * model * vec4(pos, 1.0);
}
