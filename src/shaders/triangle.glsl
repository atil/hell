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

    v2f_normal = normalize(mat3(transpose(inverse(u_model))) * in_normal);  

    v2f_tex_coord = in_tex_coord;
    gl_Position = u_projection * u_view * u_model * vec4(in_position, 1.0);
}
#endif

#ifdef FRAGMENT
uniform sampler2D u_texture0;
uniform sampler2D u_shadowmap;
uniform vec3 u_light_pos;
uniform mat4 u_light_v; // TODO #PERF: Merge these two
uniform mat4 u_light_p;
uniform vec4 u_light_color;

in vec3 v2f_frag_world_pos;
in vec4 v2f_frag_light_space_pos;
in vec2 v2f_tex_coord;
in vec3 v2f_normal;

out vec4 out_color;

float shadow_calc(vec4 frag_light_space_pos) {
    vec3 pos = frag_light_space_pos.xyz * 0.5 + 0.5;
    pos.z = min(pos.z, 1.0);

    float depth = texture(u_shadowmap, pos.xy).r;

    float bias = 0.005;
    return (depth + bias) < pos.z ? 0.0 : 1.0; // 0 if shadowed
}

void main()
{
    vec4 tex_color = texture(u_texture0, v2f_tex_coord);

    vec3 light_dir = normalize(u_light_pos);

    float diff = max(dot(v2f_normal, light_dir), 0.0);
    tex_color = vec4(tex_color.rgb * (diff + 0.1), 1.0);

    float shadow = shadow_calc(v2f_frag_light_space_pos);
    vec4 shadowed_tex_color = vec4(tex_color.rgb * 0.2, 1.0);

    out_color = (1.0 - shadow) * shadowed_tex_color + shadow * tex_color;
}
#endif
