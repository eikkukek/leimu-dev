#version 450

layout(set = 0, binding = 0) uniform Buffer {
    mat4 mv;
} buf;

layout(push_constant) uniform Pc {
    mat4 p;
} pc;

layout(location = 0) in vec3 in_pos;

void main() {
    gl_Position = buf.mv * pc.p * vec4(in_pos, 1.0);
}
