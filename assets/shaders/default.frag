#version 330 core
in vec3 v_pos;
in vec2 v_uv;
in vec3 v_normal;

uniform vec3 color;
uniform sampler2D tex;
uniform bool use_tex;
uniform float spec;
uniform float metallic;

layout(location = 0) out vec4 galbedo;
layout(location = 1) out vec4 gposition;
layout(location = 2) out vec4 gnormal;
layout(location = 3) out vec4 gmaterial;

void main() {
	galbedo = vec4(color, 1.0);
	if (use_tex) {
		galbedo *= texture(tex, v_uv);
	}
	gposition = vec4(v_pos, 1.0);
	gnormal = vec4(v_normal, 1.0);
	gmaterial = vec4(spec, metallic, 0.0, 0.0);
}
