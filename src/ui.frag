#version 330 core

uniform sampler2D texture_ui;

in VS_OUTPUT {
    vec2 TexCoord;
} IN;

out vec4 Color;

void main()
{
    Color = texture(texture_ui, IN.TexCoord);
}
