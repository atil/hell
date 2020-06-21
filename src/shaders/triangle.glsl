#ifdef VERTEX
layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_tex_coord;
layout (location = 2) in vec3 in_normal;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;
uniform mat4 u_light_v;
uniform mat4 u_light_p;

out vec3 v2f_frag_world_pos;
out vec4 v2f_frag_light_space_pos;
out vec2 v2f_tex_coord;
out vec3 v2f_normal;

#define JITTER_RESOLUTION vec2(160, 120)

void main()
{
    v2f_frag_world_pos = vec3(u_model * vec4(in_position, 1.0));
    v2f_frag_light_space_pos = u_light_p * u_light_v * vec4(v2f_frag_world_pos, 1.0);

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

uniform vec3 u_light_dir;
uniform mat4 u_light_v; // TODO #PERF: Merge these two
uniform mat4 u_light_p;
uniform vec4 u_light_color;
uniform vec3 u_point_light_pos;
uniform float u_point_light_intensity;
uniform float u_point_light_attenuation;
uniform float u_far_plane;

in vec3 v2f_frag_world_pos;
in vec4 v2f_frag_light_space_pos;
in vec2 v2f_tex_coord;
in vec3 v2f_normal;

out vec4 out_color;

#define SAMPLE_SIZE 1

float is_in_shadow_directional() {
    vec3 light_dir = -normalize(u_light_dir);
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
            shadow += (pcf_depth + bias) < pos.z ? 0.0 : 1.0; // 0 if shadowed
        }    
    }
    return shadow / ((SAMPLE_SIZE + 2) * (SAMPLE_SIZE + 2));
}

float is_in_shadow_point() {

    // get vector between fragment position and light position
    vec3 light_to_frag = v2f_frag_world_pos - u_point_light_pos;

    // ise the fragment to light vector to sample from the depth map    
    float closest_depth = texture(u_shadowmap_point, light_to_frag).r;

    // it is currently in linear range between [0,1], let's re-transform it back to original depth value
    closest_depth *= u_far_plane;
    // now get current linear depth as the length between the fragment and light position
    float current_depth = length(light_to_frag);
    float bias = 0.05; // we use a much larger bias since depth is now in [near_plane, far_plane] range
    return current_depth - bias > closest_depth ? 1.0 : 0.0; // 1 if shadowed

}
 
float get_frag_brightness() {
    vec3 frag_to_directional_light = normalize(-u_light_dir);
    float alignment_with_directional_light = max(dot(v2f_normal, frag_to_directional_light), 0.0);

    vec3 frag_to_point_light = normalize(u_point_light_pos - v2f_frag_world_pos);
    float alignment_with_point_light = max(dot(v2f_normal, frag_to_point_light), 0.0);
    float distance_to_point_light = distance(u_point_light_pos, v2f_frag_world_pos);

    float point_light_brightness = alignment_with_point_light * u_point_light_intensity / (u_point_light_attenuation * distance_to_point_light);

    return max(point_light_brightness, 0.1);
}

void main() {
    vec4 tex_color = texture(u_texture0, v2f_tex_coord);

    tex_color = vec4(tex_color.rgb * get_frag_brightness(), 1.0);

    /* float shadow = is_in_shadow_directional(); */
    /* shadow += is_in_shadow_point(); */

    float shadow = is_in_shadow_point();

    vec4 shadowed_tex_color = vec4(tex_color.rgb * 0.2, 1.0);

    out_color = (1.0 - shadow) * tex_color + shadow * shadowed_tex_color;
}
#endif
