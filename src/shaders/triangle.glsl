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

#define JITTER_EFFECT
#define JITTER_RESOLUTION vec2(160, 120)
#
void main()
{
    v2f_frag_world_pos = vec3(u_model * vec4(in_position, 1.0));
    v2f_frag_light_space_pos = u_light_p * u_light_v * vec4(v2f_frag_world_pos, 1.0);

    v2f_normal = normalize(mat3(transpose(inverse(u_model))) * in_normal);  

    v2f_tex_coord = in_tex_coord;

    vec4 clip_pos = u_projection * u_view * u_model * vec4(in_position, 1.0);

#ifdef JITTER_EFFECT
    clip_pos.xyz = clip_pos.xyz / clip_pos.w; // clip space -> NDC
    clip_pos.xy = floor(JITTER_RESOLUTION * clip_pos.xy) / JITTER_RESOLUTION;
    clip_pos.xyz *= clip_pos.w; // NDC -> clip space
#endif

    gl_Position = clip_pos;
}
#endif

#ifdef FRAGMENT
uniform sampler2D u_texture0;
uniform sampler2D u_shadowmap;
uniform vec3 u_light_dir;
uniform mat4 u_light_v; // TODO #PERF: Merge these two
uniform mat4 u_light_p;
uniform vec4 u_light_color;

in vec3 v2f_frag_world_pos;
in vec4 v2f_frag_light_space_pos;
in vec2 v2f_tex_coord;
in vec3 v2f_normal;

out vec4 out_color;

#define SAMPLE_SIZE 1

float shadow_calc(vec4 frag_light_space_pos, vec3 light_dir) {
    vec3 pos = frag_light_space_pos.xyz * 0.5 + 0.5;
    pos.z = min(pos.z, 1.0);

    float depth = texture(u_shadowmap, pos.xy).r;

    // If the surface is perpendicular to the light direction
    // then it needs larger bias values
    float bias = max(0.002 * (1.0 - dot(v2f_normal, light_dir)), 0.00001);

    float shadow = 0.0;
    vec2 texel_size = 1.0 / textureSize(u_shadowmap, 0);
    for(int x = -SAMPLE_SIZE; x <= SAMPLE_SIZE; x++) {
        for(int y = -SAMPLE_SIZE; y <= SAMPLE_SIZE; y++) {
            float pcf_depth = texture(u_shadowmap, pos.xy + vec2(x, y) * texel_size).r; 
            shadow += (pcf_depth + bias) < pos.z ? 0.0 : 1.0; // 0 if shadowed
        }    
    }
    return shadow / ((SAMPLE_SIZE + 2) * (SAMPLE_SIZE + 2));
}

void main() {
    vec4 tex_color = texture(u_texture0, v2f_tex_coord);

    vec3 frag_to_directional_light = normalize(-u_light_dir);
    float diff = max(dot(v2f_normal, frag_to_directional_light), 0.0);
    tex_color = vec4(tex_color.rgb * (diff + 0.1), 1.0);

    float shadow = shadow_calc(v2f_frag_light_space_pos, frag_to_directional_light);
    vec4 shadowed_tex_color = vec4(tex_color.rgb * 0.2, 1.0);

    out_color = (1.0 - shadow) * shadowed_tex_color + shadow * tex_color;
}
#endif
