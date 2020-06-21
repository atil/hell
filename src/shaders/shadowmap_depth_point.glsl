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

out vec4 g2f_frag_pos; // FragPos from GS (output per emitvertex)

void main()
{
    for(int face = 0; face < 6; ++face)
    {
        gl_Layer = face; // built-in variable that specifies to which face we render.
        for(int i = 0; i < 3; ++i) // for each triangle's vertices
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
