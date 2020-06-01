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

void main()
{
    v2f_frag_world_pos = vec3(u_model * vec4(in_position, 1.0));
    v2f_frag_light_space_pos = u_light_p * u_light_v * vec4(v2f_frag_world_pos, 1.0);

    v2f_normal = mat3(transpose(inverse(u_model))) * in_normal;  

    v2f_tex_coord = in_tex_coord;
    gl_Position = u_projection * u_view * u_model * vec4(in_position, 1.0);
}
#endif

#ifdef FRAGMENT
uniform sampler2D u_texture0;
uniform sampler2D u_shadowmap;
uniform mat4 u_light_v;
uniform mat4 u_light_p;

in vec3 v2f_frag_world_pos;
in vec4 v2f_frag_light_space_pos;
in vec2 v2f_tex_coord;
in vec3 v2f_normal;

out vec4 out_color;

float shadow_calc(vec4 frag_pos_light_space)
{
    vec3 proj_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;

    proj_coords = proj_coords * 0.5 + 0.5;

    float closest_depth = texture(u_shadowmap, proj_coords.xy).r; 

    float current_depth = min(proj_coords.z, 1.0);

    float shadow = current_depth > closest_depth  ? 1.0 : 0.0;

    return shadow; // 1.0 if in shadow
}  

float shadow_calc2(vec4 frag_light_space_pos) {
    vec3 pos = frag_light_space_pos.xyz * 0.5 + 0.5;
    pos.z = min(pos.z, 1.0);

    float depth = texture(u_shadowmap, pos.xy).r;

    return (depth + 0.005) < pos.z ? 0.0 : 1.0; // 0 if shadowed
}

void main()
{
    vec4 tex_color = texture(u_texture0, v2f_tex_coord);

    vec3 light_dir = normalize(vec3(1.0, 1.0, 0.0)); // TODO #CLEANUP: Calculate this from the light position

    vec3 norm = normalize(v2f_normal);
    float diff = max(dot(norm, light_dir), 0.0);
    vec3 light_color = vec3(0.2, 0.1, 0.0);

    vec4 diffuse = vec4(diff * light_color, 1.0);

    float shadow = shadow_calc2(v2f_frag_light_space_pos);
    vec4 shadowed_tex_color = vec4(tex_color.rgb * 0.2, 1.0);

    out_color = (1.0 - shadow) * shadowed_tex_color + shadow * tex_color;
}
#endif
