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

    float alignment_with_directional_light = dot(v2f_normal, light_dir);
    if (alignment_with_directional_light < 0.0) {
        // Surface looking away from light is always in shadow
        return 1.0;
    }

    vec3 pos = v2f_frag_light_space_pos.xyz * 0.5 + 0.5;
    pos.z = min(pos.z, 1.0);

    // If the surface is perpendicular to the light direction
    // then it needs larger bias values
    float bias = max(0.0005 * (1.0 - alignment_with_directional_light), 0.00001);

    float shadow = 0.0;

    // Uncomment for hard directional shadows
    /* float pcf_depth = texture(u_shadowmap_directional, pos.xy ).r; */ 
    /* shadow += (pcf_depth + bias) < pos.z ? 1.0 : 0.0; // 1 if shadowed */
    /* return shadow; */

    vec2 texel_size = 1.0 / textureSize(u_shadowmap_directional, 0);
    for (int x = -DIRECTIONAL_LIGHT_SAMPLES; x <= DIRECTIONAL_LIGHT_SAMPLES; x++) {
        for (int y = -DIRECTIONAL_LIGHT_SAMPLES; y <= DIRECTIONAL_LIGHT_SAMPLES; y++) {
            float pcf_depth = texture(u_shadowmap_directional, pos.xy + vec2(x, y) * texel_size).r; 
            shadow += (pcf_depth + bias) < pos.z ? 1.0 : 0.0; // 1 if shadowed
        }    
    }
    return shadow / ((DIRECTIONAL_LIGHT_SAMPLES + 2) * (DIRECTIONAL_LIGHT_SAMPLES + 2));
} 

vec3 point_shadow_sample_offset_directions[20] = vec3[] (
   vec3( 1,  1,  1), vec3( 1, -1,  1), vec3(-1, -1,  1), vec3(-1,  1,  1), 
   vec3( 1,  1, -1), vec3( 1, -1, -1), vec3(-1, -1, -1), vec3(-1,  1, -1),
   vec3( 1,  1,  0), vec3( 1, -1,  0), vec3(-1, -1,  0), vec3(-1,  1,  0),
   vec3( 1,  0,  1), vec3(-1,  0,  1), vec3( 1,  0, -1), vec3(-1,  0, -1),
   vec3( 0,  1,  1), vec3( 0, -1,  1), vec3( 0, -1, -1), vec3( 0,  1, -1)
);  

#define POINT_SHADOW_SOFTNESS_WITH_DISTANCE 0.005

float get_frag_brightness_from_light(PointLight light, int light_index) {
    float t_shadow = 0.0;
    vec3 light_to_frag = v2f_frag_world_pos - light.position;
    float light_to_frag_dist = length(light_to_frag) - 0.0005; // bias

    // Larger sample distance as the fragment is further away from the light
    float disk_radius = POINT_SHADOW_SOFTNESS_WITH_DISTANCE * light_to_frag_dist;
    for (int i = 0; i < 20; i++) {
        vec3 sample_dir = light_to_frag + point_shadow_sample_offset_directions[i] * disk_radius;
        float depth_in_cubemap = texture(u_shadowmaps_point, vec4(sample_dir, light_index)).r;
        depth_in_cubemap *= u_far_plane;
        if (light_to_frag_dist > depth_in_cubemap) {
            t_shadow += 1.0;
        }
    }
    t_shadow = t_shadow / 20.0;

    vec3 frag_to_point_light = light.position - v2f_frag_world_pos;
    float alignment_with_point_light = max(dot(v2f_normal, normalize(frag_to_point_light)), 0.0);
    float distance_to_point_light = length(frag_to_point_light);

    float max_brightness = alignment_with_point_light * light.intensity
        / (light.attenuation * distance_to_point_light);

    return mix(max_brightness, 0.0, t_shadow);
}

void main() {
    vec4 tex_color = texture(u_texture0, v2f_tex_coord);

    vec4 shadowed_tex_color = vec4(tex_color.rgb * 0.05, 1.0);

    float brightness = 0.0;
    brightness += mix(0.2, 0.0, get_t_shadow_directional());

    for (int i = 0; i < u_point_light_count; i++) {
        brightness += get_frag_brightness_from_light(u_point_lights[i], i);
    }

    out_color = mix(shadowed_tex_color, tex_color, brightness);
}
#endif
