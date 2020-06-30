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

#define LIGHT_COUNT 1

uniform PointLight u_point_lights[LIGHT_COUNT];

/* uniform vec3 u_point_light_pos; */
/* uniform float u_point_light_intensity; */
/* uniform float u_point_light_attenuation; */

uniform float u_far_plane;

in vec3 v2f_frag_world_pos;
in vec4 v2f_frag_light_space_pos;
in vec2 v2f_tex_coord;
in vec3 v2f_normal;

out vec4 out_color;

#define SAMPLE_SIZE 1

float is_in_shadow_directional() {
    vec3 light_dir = -normalize(u_directional_light_dir);
    vec3 pos = v2f_frag_light_space_pos.xyz * 0.5 + 0.5;
    pos.z = min(pos.z, 1.0);

    // If the surface is perpendicular to the light direction
    // then it needs larger bias values
    float bias = max(0.002 * (1.0 - dot(v2f_normal, light_dir)), 0.00001);

    float shadow = 0.0;
    vec2 texel_size = 1.0 / textureSize(u_shadowmap_directional, 0);
    for(int x = -SAMPLE_SIZE; x <= SAMPLE_SIZE; x++) {
        for(int y = -SAMPLE_SIZE; y <= SAMPLE_SIZE; y++) {
            float pcf_depth = texture(u_shadowmap_directional, pos.xy + vec2(x, y) * texel_size).r; 
            shadow += (pcf_depth + bias) < pos.z ? 1.0 : 0.0; // 1 if shadowed
        }    
    }
    return shadow / ((SAMPLE_SIZE + 2) * (SAMPLE_SIZE + 2));
}

float is_in_shadow_point() {
    float is_shadow = 0;
    for (int i = 0; i < LIGHT_COUNT; i++) {
        vec3 light_to_frag = v2f_frag_world_pos - u_point_lights[i].position;

        float depth_in_cubemap = texture(u_shadowmaps_point, vec4(light_to_frag, i)).r;
        depth_in_cubemap *= u_far_plane;

        float bias = 0.05;
        // TODO Soft shadows: Sample the nearby cube-texels
        is_shadow += length(light_to_frag) - bias > depth_in_cubemap ? 1.0 : 0.0; // 1 if shadowed
    }
    return clamp(is_shadow, 0, 1);

    /* vec3 light_to_frag = v2f_frag_world_pos - u_point_light_pos; */

    /* float depth_in_cubemap = texture(u_shadowmap_point, light_to_frag).r; */
    /* depth_in_cubemap *= u_far_plane; */

    /* float bias = 0.05; */
    /* // TODO Soft shadows: Sample the nearby cube-texels */
    /* return length(light_to_frag) - bias > depth_in_cubemap ? 1.0 : 0.0; // 1 if shadowed */
}
 
float get_frag_brightness() {
    vec3 frag_to_directional_light = normalize(-u_directional_light_dir);
    float alignment_with_directional_light = max(dot(v2f_normal, frag_to_directional_light), 0.0);

    float point_light_brightness = 0;
    for (int i = 0; i < LIGHT_COUNT; i++) {
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

    float shadow = is_in_shadow_directional();
    shadow += is_in_shadow_point();

    vec4 shadowed_tex_color = vec4(tex_color.rgb * 0.2, 1.0);

    out_color = (1.0 - shadow) * tex_color + shadow * shadowed_tex_color;
}
#endif
