#version 150

in vec2 pos;
in vec2 texpos;

uniform mat4 proj, view, model;

out vec2 f_texpos;

void main() {
    f_texpos = texpos;
    gl_Position = proj * view * model * vec4(pos, 0, 1);
}
