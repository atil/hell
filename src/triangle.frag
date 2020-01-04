#version 330 core

in VS_OUTPUT {
    vec3 Color;
} IN;

out vec4 Color;

void main()
{
    Color = vec4(0.2, 0.5, 0.7, 1.0f);
}