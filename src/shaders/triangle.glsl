#ifdef VERTEX
layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_tex_coord;
layout (location = 2) in vec3 in_normal;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 v2f_frag_world_pos;
out vec2 v2f_tex_coord;
out vec3 v2f_normal;

void main()
{
    v2f_frag_world_pos = vec3(u_model * vec4(in_position, 1.0));
    v2f_normal = mat3(transpose(inverse(u_model))) * in_normal;  

    v2f_tex_coord = vec2(in_tex_coord.x, in_tex_coord.y);
    gl_Position = u_projection * u_view * u_model * vec4(in_position, 1.0);
}
#endif

#ifdef FRAGMENT
uniform sampler2D u_texture0;
uniform vec3 u_light_dir;

in vec3 v2f_frag_world_pos;
in vec2 v2f_tex_coord;
in vec3 v2f_normal;

out vec4 out_color;

void main()
{
    //// vec3 lightDir = normalize(lightPos - frag_world_pos);
    vec4 tex_color = texture(u_texture0, v2f_tex_coord);

    vec3 norm = normalize(v2f_normal);
    float diff = max(dot(norm, -u_light_dir), 0.0);
    vec3 light_color = vec3(0.2, 0.1, 0.0);
    vec3 diffuse = diff * light_color;

    out_color = tex_color * 0.5 + vec4(diffuse, 1.0);
}
#endif
