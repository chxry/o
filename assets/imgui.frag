#version 330 core
in vec2 frag_uv;
in vec4 frag_color;

uniform sampler2D tex;

out vec4 f_color;

void main() {
    vec3 col = pow(frag_color.rgb,vec3(2.2));
    f_color = vec4(col.rgb,frag_color.a) * texture(tex,frag_uv);
}