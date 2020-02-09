#version 330 core

uniform vec4 diffuse;
uniform sampler2D texture0;

in VS_OUTPUT {
    vec2 TexCoord;
} IN;

out vec4 Color;

void main()
{
    // Color = diffuse;
    Color = texture(texture0, IN.TexCoord);
}