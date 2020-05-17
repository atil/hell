#ifdef VERTEX
layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 aTexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    gl_Position = projection * view * model * vec4(Position, 1.0);
}
#endif

#ifdef FRAGMENT
uniform vec3 color0;

out vec4 OutColor;

void main()
{
    OutColor = vec4(color0, 1); // why don't we have a color here?
}
#endif
