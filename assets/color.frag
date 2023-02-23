#version 330 core
in vec3 v_pos;
in vec3 v_normal;

uniform vec3 color;
uniform sampler2D shadow_map;
uniform vec3 light_dir;
uniform mat4 light_view;
uniform mat4 light_projection;

out vec4 f_color;

float lighting() {
	float ambient = 0.1;
 	float diffuse = max(dot(normalize(v_normal), normalize(light_dir)), 0.0);
	
	vec4 lightspace = light_projection * light_view * vec4(v_pos, 1.0);
    lightspace = lightspace * 0.5 + 0.5;
    float current = lightspace.z - 0.0015;
    float shadow = 0.0;
    float closest = texture(shadow_map, lightspace.xy).r;
    vec2 texel_size = 1.0 / textureSize(shadow_map, 0);
	int shadow_softness = 2;
    for (int x = -shadow_softness; x <= shadow_softness; ++x) {
      for (int y = -shadow_softness; y <= shadow_softness; ++y) {
        float closest = texture(shadow_map, lightspace.xy + vec2(x, y) * texel_size).r;
        shadow += current > closest ? 1.0 : 0.0;
      }
    }
    shadow /= pow(shadow_softness * 2 + 1, 2);
	return ambient + (1.0 - shadow) * diffuse;
}

void main() {
	f_color = vec4(lighting() * color, 1.0);
}
