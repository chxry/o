#version 450 core
layout (location = 0) in vec2 a_uv;
layout (location = 1) in vec4 a_color;

layout (binding = 0) uniform sampler2D u_texture;

layout (location = 0) out vec4 f_color;

void main() {
    vec4 col = a_color * texture(u_texture, a_uv);
    f_color = vec4(pow(col.rgb,vec3(2.2)),col.w);
}