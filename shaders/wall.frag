in vec2 v_uv;

uniform sampler2D albedo;

out vec4 frag;

void main() {
  frag.rgb += texture(albedo, v_uv).rgb;
  frag.a = 1;
}
