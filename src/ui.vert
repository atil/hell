#version 330 core
layout (location = 0) in vec2 vertex;
layout (location = 1) in vec2 aTexCoord;

out VS_OUTPUT {
    vec2 TexCoord;
} OUT;

void main()
{
    gl_Position = vec4(vertex, 0.0, 1.0);
    OUT.TexCoord = vec2(aTexCoord.x, aTexCoord.y);
}
