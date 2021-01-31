uniform mat4 view;
uniform mat4 projection;

out vec2 uv;

void main() {
  int y = gl_VertexID / 2;
  int x = (gl_VertexID + y) % 2;

  vec2 pos = vec2(x, y);
  gl_Position = projection * view * vec4(pos - 0.5, 0., 1.);
  uv = vec2(x, 1 - y);
}
