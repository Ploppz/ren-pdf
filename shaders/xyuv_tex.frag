#version 150

in vec2 f_texpos;

uniform sampler2D tex;

out vec4 color;

void main() {
    float intensity = texture(tex, f_texpos).r;
    color = intensity * vec4(1,1,1,1);
}
