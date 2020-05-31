#ifdef VERTEX

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec2 in_texcoords;

out vec2 v2f_texcoords;

void main()
{
    v2f_texcoords = in_texcoords;
    gl_Position = vec4(in_pos, 1.0);
}

#endif

#ifdef FRAGMENT
out vec4 out_color;

in vec2 v2f_texcoords;

uniform sampler2D u_depth_map;

void main()
{             
    float depth_value = texture(u_depth_map, v2f_texcoords).r;
    out_color = vec4(vec3(depth_value), 1.0); // orthographic
}
#endif
