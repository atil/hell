#ifdef VERTEX
layout (location = 0) in vec3 in_position;

uniform mat4 u_light_v;
uniform mat4 u_light_p;
uniform mat4 u_model;

void main()
{
    gl_Position = u_light_p * u_light_v * u_model * vec4(in_position, 1.0);
}
#endif

#ifdef FRAGMENT
void main() {}
#endif
