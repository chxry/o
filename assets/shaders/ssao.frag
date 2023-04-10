#version 330 core
in vec2 uv;

uniform sampler2D galbedo;
uniform sampler2D gposition;
uniform sampler2D gnormal;
uniform sampler2D noise;
uniform vec3 samples[64];
uniform mat4 view;
uniform mat4 projection;

float radius = 0.7;
float bias = 0.025;

out float f_color;

void main() {
	if (texture(galbedo, uv).a > 0) {
		vec3 pos =  (view * texture(gposition, uv)).xyz;
		vec3 normal = normalize((inverse(transpose(view)) * texture(gnormal, uv)).xyz);
		vec3 rand = normalize(texture(noise, uv * 400.0).xyz);
		
		vec3 tangent = normalize(rand - normal * dot(rand, normal));
		vec3 bitangent = cross(normal, tangent);
		mat3 TBN = mat3(tangent, bitangent, normal);

		float occlusion = 0.0;
		for(int i = 0; i < 64; i++) {
			vec3 sample_pos = TBN * samples[i];
			sample_pos = pos + sample_pos * radius;
		
	        vec4 projected = projection *  vec4(sample_pos, 1.0);
			projected.xy /= projected.w;
			projected.xy = projected.xy * 0.5 + 0.5;
		
			float depth = (view * texture(gposition, projected.xy)).z;
			float range_check = smoothstep(0.0, 1.0, radius / abs(pos.z - depth));
			occlusion += (depth >= sample_pos.z + bias ? 1.0 : 0.0) * range_check;
		}
		f_color = 1.0 - (occlusion / 64.0);
	}
}
