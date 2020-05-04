#version 330 core

uniform sampler2D texture0;

in VS_OUTPUT {
    vec2 TexCoord;
} IN;

out vec4 Color;

void main()
{
    Color = texture(texture0, IN.TexCoord);
}
