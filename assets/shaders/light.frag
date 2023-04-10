#version 330 core
in vec2 uv;

uniform sampler2D galbedo;
uniform sampler2D gposition;
uniform sampler2D gnormal;
uniform sampler2D gmaterial;
uniform sampler2D ssao_tex;
uniform sampler2D shadow_map;
uniform mat4 view;
uniform mat4 projection;
uniform vec3 cam_pos;
uniform vec3 sun_dir;
uniform mat4 sun_view;
uniform mat4 sun_projection;

struct light_t {
	vec3 pos;
	vec3 color;
	float strength;
};
uniform light_t lights[100];
uniform int num_lights;

out vec4 f_color;

vec2 raymarch(vec3 pos, vec3 dir) {
	dir *= 0.05;
	vec4 projected;
	for(int i = 0; i < 400; i++) {
		pos += dir;
		
        projected = projection * vec4(pos, 1.0);
		projected.xy /= projected.w;
		projected.xy = projected.xy * 0.5 + 0.5;

		float depth = (view * texture(gposition, projected.xy)).z;
		float diff = pos.z - depth;
		if (diff <= 0.0 && dir.z - diff < 1.2) {
			for (int i = 0; i < 5; i++) {
				projected = projection * vec4(pos, 1.0);
				projected.xy /= projected.w;
				projected.xy = projected.xy * 0.5 + 0.5;
				float depth = (view * texture(gposition, projected.xy)).z;
				float diff = pos.z - depth;
				dir *= 0.5;
				if (diff > 0.0) {
					pos += dir;
				} else {
					pos -= dir;
				}
			}
			break;
		}
    }
	return projected.xy;
}

vec3 calc_light(vec3 pos, vec3 normal, vec3 dir, vec3 color, float spec, float atten) {
 	float diffuse = max(dot(normal, dir), 0.0);
	float specular = pow(max(dot(normalize(cam_pos - pos), normalize(-reflect(dir, normal))), 0.0), 16.0) * spec;
	diffuse *= atten;
	specular *= atten;
	return vec3(diffuse + specular) * color;
}

void main(){
	vec4 albedo = texture(galbedo, uv);
	if (albedo.a > 0) {
		vec3 pos = texture(gposition,uv).xyz;
		vec3 view_pos =  (view * vec4(pos, 1.0)).xyz;
		vec3 normal = normalize(texture(gnormal, uv).xyz);
		vec3 view_normal = (mat3(view) * normal).xyz;
		vec4 material = texture(gmaterial, uv);
		float spec = material.x;
		float metallic = material.y;
		vec3 reflected = normalize(reflect(normalize(view_pos), view_normal));

		vec3 light = calc_light(pos, normal, sun_dir, albedo.rgb, spec, 1.0);
		
		vec4 light_pos = sun_projection * sun_view * vec4(pos, 1.0);
	    light_pos = light_pos * 0.5 + 0.5;
	    float current = light_pos.z - 0.0015;
	    float shadow = 0.0;
	    float closest = texture(shadow_map, light_pos.xy).r;
	    vec2 texel_size = 1.0 / textureSize(shadow_map, 0);
		int shadow_softness = 2;
	    for (int x = -shadow_softness; x <= shadow_softness; ++x) {
	      for (int y = -shadow_softness; y <= shadow_softness; ++y) {
	        float closest = texture(shadow_map, light_pos.xy + vec2(x, y) * texel_size).r;
	        shadow += current > closest ? 1.0 : 0.0;
	      }
	    }
		 shadow /= pow(shadow_softness * 2 + 1, 2);
		light *= (1 - vec3(shadow));
		
		light += albedo.rgb * 0.1;
		for (int i = 0; i < num_lights; i++) {
			vec3 dir = lights[i].pos - pos;
			float distance = length(dir);
			light += calc_light(pos,normal,dir, lights[i].color * albedo.rgb, spec, 1.0 / (pow(distance / lights[i].strength, 2.0) + 1.0));
		}
		light *= texture(ssao_tex, uv).r;
	
     	vec2 coords = raymarch(view_pos, reflected);
		float reflection_multiplier = clamp(pow(metallic, 3) * -reflected.z, 0.0, 0.9);
		f_color = vec4(light + texture(galbedo, coords).rgb * reflection_multiplier, 1.0);
	} else {
		f_color	= vec4(albedo.rgb, 1.0);
	}
}
