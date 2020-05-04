#version 330 core

uniform sampler2D texture0;

in VS_OUTPUT {
    vec2 TexCoord;
} IN;

out vec4 Color;

void main()
{
    // Color = vec4(1, 0, 0.5, 1);
    Color = texture(texture0, IN.TexCoord);
}
