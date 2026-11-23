#version 450

layout(set = 0, binding = 1) uniform Buffer {
    vec4 color;
} buf;

layout(push_constant) uniform Pc {
    layout(offset = 64) vec4 color;
} pc;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = buf.color + pc.color;
}
