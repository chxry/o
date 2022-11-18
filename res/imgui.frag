#version 330 core
in vec2 frag_uv;
in vec4 frag_color;

uniform sampler2D tex;

out vec4 f_color;

void main() {
    vec4 col = frag_color * texture(tex, frag_uv);
    f_color = vec4(pow(col.rgb,vec3(2.2)),col.w);
}