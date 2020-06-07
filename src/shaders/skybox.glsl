#ifdef VERTEX
layout (location = 0) in vec3 in_pos;

uniform mat4 u_projection;
uniform mat4 u_view;

out vec3 v2f_texcoord;

void main()
{
    v2f_texcoord = in_pos;
    vec4 pos = u_projection * u_view * vec4(in_pos, 1.0);
    gl_Position = pos.xyww;
}  
#endif

#ifdef FRAGMENT
uniform samplerCube u_skybox;

in vec3 v2f_texcoord;

out vec4 out_frag_color;

void main()
{    
    out_frag_color = texture(u_skybox, v2f_texcoord);
}
#endif 
