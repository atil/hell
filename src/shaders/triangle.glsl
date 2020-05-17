#ifdef VERTEX
layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 aTexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out VS_OUTPUT {
    vec2 TexCoord;
} OUT;

void main()
{
    gl_Position = projection * view * model * vec4(Position, 1.0);
    OUT.TexCoord = vec2(aTexCoord.x, aTexCoord.y);
}
#endif

#ifdef FRAGMENT
uniform sampler2D texture0;

in VS_OUTPUT {
    vec2 TexCoord;
} IN;

out vec4 Color;

void main()
{
    Color = texture(texture0, IN.TexCoord);
}
#endif
