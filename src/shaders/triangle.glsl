#ifdef VERTEX
layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_tex_coord;
layout (location = 2) in vec3 in_normal;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;
uniform mat4 u_directional_light_vp;

out vec3 v2f_frag_world_pos;
out vec4 v2f_frag_light_space_pos;
out vec2 v2f_tex_coord;
out vec3 v2f_normal;

#define JITTER_RESOLUTION vec2(160, 120)

void main()
{
    v2f_frag_world_pos = vec3(u_model * vec4(in_position, 1.0));
    v2f_frag_light_space_pos = u_directional_light_vp* vec4(v2f_frag_world_pos, 1.0);

    v2f_normal = normalize(mat3(transpose(inverse(u_model))) * in_normal);  

    v2f_tex_coord = in_tex_coord;

    vec4 clip_pos = u_projection * u_view * u_model * vec4(in_position, 1.0);

    // jitter effect
    clip_pos.xyz = clip_pos.xyz / clip_pos.w; // clip space -> NDC
    clip_pos.xy = floor(JITTER_RESOLUTION * clip_pos.xy) / JITTER_RESOLUTION;
    clip_pos.xyz *= clip_pos.w; // NDC -> clip space

    gl_Position = clip_pos;
}
#endif

#ifdef FRAGMENT
uniform sampler2D u_texture0;
uniform sampler2D u_shadowmap_directional;
uniform samplerCube u_shadowmap_point;

uniform vec3 u_directional_light_dir;
uniform mat4 u_directional_light_vp;
uniform vec4 u_directional_light_color;

struct PointLight {
    vec3 position;
    float intensity;
    float attenuation;
};

uniform samplerCubeArray u_shadowmaps_point;

#define MAX_LIGHT_COUNT 2

uniform PointLight u_point_lights[MAX_LIGHT_COUNT];
uniform int u_point_light_count;
uniform float u_far_plane;

in vec3 v2f_frag_world_pos;
in vec4 v2f_frag_light_space_pos;
in vec2 v2f_tex_coord;
in vec3 v2f_normal;

out vec4 out_color;

#define DIRECTIONAL_LIGHT_SAMPLES 1

float get_t_shadow_directional() {
    vec3 light_dir = -normalize(u_directional_light_dir);
    vec3 pos = v2f_frag_light_space_pos.xyz * 0.5 + 0.5;
    pos.z = min(pos.z, 1.0);

    // If the surface is perpendicular to the light direction
    // then it needs larger bias values
    float bias = max(0.002 * (1.0 - dot(v2f_normal, light_dir)), 0.00001);

    float shadow = 0.0;
    vec2 texel_size = 1.0 / textureSize(u_shadowmap_directional, 0);
    for(int x = -DIRECTIONAL_LIGHT_SAMPLES; x <= DIRECTIONAL_LIGHT_SAMPLES; x++) {
        for(int y = -DIRECTIONAL_LIGHT_SAMPLES; y <= DIRECTIONAL_LIGHT_SAMPLES; y++) {
            float pcf_depth = texture(u_shadowmap_directional, pos.xy + vec2(x, y) * texel_size).r; 
            shadow += (pcf_depth + bias) < pos.z ? 1.0 : 0.0; // 1 if shadowed
        }    
    }
    return shadow / ((DIRECTIONAL_LIGHT_SAMPLES + 2) * (DIRECTIONAL_LIGHT_SAMPLES + 2));
} 

vec3 sample_offset_directions[20] = vec3[]
(
   vec3( 1,  1,  1), vec3( 1, -1,  1), vec3(-1, -1,  1), vec3(-1,  1,  1), 
   vec3( 1,  1, -1), vec3( 1, -1, -1), vec3(-1, -1, -1), vec3(-1,  1, -1),
   vec3( 1,  1,  0), vec3( 1, -1,  0), vec3(-1, -1,  0), vec3(-1,  1,  0),
   vec3( 1,  0,  1), vec3(-1,  0,  1), vec3( 1,  0, -1), vec3(-1,  0, -1),
   vec3( 0,  1,  1), vec3( 0, -1,  1), vec3( 0, -1, -1), vec3( 0,  1, -1)
);  

float get_t_shadow_point() {

    float shadow_amount = 0.0;

    for (int i = 0; i < u_point_light_count; i++) {
        vec3 light_to_frag = v2f_frag_world_pos - u_point_lights[i].position;
        float light_to_frag_dist = length(light_to_frag) - 0.0005;

        // Larger sample distance as the fragment is further away from the light
        float disk_radius = 0.002;// * light_to_frag_dist;

        for (int i = 0; i < 20; i++) {
            vec3 sample_dir = light_to_frag + sample_offset_directions[i] * disk_radius;
            float depth_in_cubemap = texture(u_shadowmaps_point, vec4(sample_dir, i)).r;
            depth_in_cubemap *= u_far_plane;
            if (light_to_frag_dist > depth_in_cubemap) {
                shadow_amount += 1;
            }
        }

    }

    return shadow_amount / (float(20) * u_point_light_count);
}
 
float get_frag_brightness() {
    vec3 frag_to_directional_light = normalize(-u_directional_light_dir);
    float alignment_with_directional_light = max(dot(v2f_normal, frag_to_directional_light), 0.0);

    float point_light_brightness = 0;
    for (int i = 0; i < u_point_light_count; i++) {

        PointLight li = u_point_lights[i];
        vec3 frag_to_point_light = normalize(li.position - v2f_frag_world_pos);
        float alignment_with_point_light = max(dot(v2f_normal, frag_to_point_light), 0.0);
        float distance_to_point_light = distance(li.position, v2f_frag_world_pos);

        point_light_brightness += alignment_with_point_light * li.intensity
            / (li.attenuation * distance_to_point_light);
    }

    return max(point_light_brightness + alignment_with_directional_light, 0.1);
}

void main() {
    vec4 tex_color = texture(u_texture0, v2f_tex_coord);

    tex_color = vec4(tex_color.rgb * get_frag_brightness(), 1.0);
    vec4 shadowed_tex_color = vec4(tex_color.rgb * 0.2, 1.0);

    float t_shadow = get_t_shadow_directional();
    t_shadow = get_t_shadow_point();

    out_color = mix(tex_color, shadowed_tex_color, t_shadow);

    /* t_shadow *= 0.001; */
    /* out_color = vec4(t_shadow, t_shadow, t_shadow, 1); */
}
#endif
