#ifdef VERTEX
layout (location = 0) in vec3 in_pos;

uniform mat4 u_model;

void main()
{
    gl_Position = u_model * vec4(in_pos, 1.0);
}
#endif

#ifdef GEOMETRY
layout (triangles) in;
layout (triangle_strip, max_vertices=18) out;

uniform mat4 u_shadow_matrices[6];
uniform int u_light_index;

out vec4 g2f_frag_pos;

void main()
{
    for (int face = 0; face < 6; face++)
    {
        gl_Layer = u_light_index * 6 + face;
        for (int i = 0; i < 3; i++)
        {
            g2f_frag_pos = gl_in[i].gl_Position;
            gl_Position = u_shadow_matrices[face] * g2f_frag_pos;
            EmitVertex();
        }    
        EndPrimitive();
    }
} 
#endif

#ifdef FRAGMENT
in vec4 g2f_frag_pos;

uniform vec3 u_light_pos;
uniform float u_far_plane;

void main()
{
    gl_FragDepth = length(g2f_frag_pos.xyz - u_light_pos) / u_far_plane;
}
#endif
