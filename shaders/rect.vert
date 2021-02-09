uniform vec2 position;
uniform vec2 size;

void main() {
    int y = gl_VertexID / 2;
    int x = (gl_VertexID + y) % 2;

    // (0, 0) is top left, (1, 1) is bottom right
    vec2 gui_pos = position + size * vec2(x, y);

    // (-1, 1) is top left, (1, -1) is bottom right
    vec2 screen_pos = vec2(gui_pos.x, -gui_pos.y) * 2 + vec2(-1, 1);

    gl_Position = vec4(screen_pos, 0, 1);
}
