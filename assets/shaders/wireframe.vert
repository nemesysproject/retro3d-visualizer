in vec3 a_pos;
in vec3 a_other;
in vec2 a_meta; // x = side (-1.0 or +1.0), y = intensity [0..1]

uniform mat4 u_view_proj;
uniform vec2 u_viewport;
uniform float u_line_width;

out float v_edge;
out float v_intensity;

void main() {
    vec4 clip_a = u_view_proj * vec4(a_pos, 1.0);
    vec4 clip_b = u_view_proj * vec4(a_other, 1.0);

    // Evitar división por cero cuando un vértice cruza el plano cercano
    float w_a = abs(clip_a.w) < 0.0001 ? 0.0001 : clip_a.w;
    float w_b = abs(clip_b.w) < 0.0001 ? 0.0001 : clip_b.w;

    vec2 ndc_a = clip_a.xy / w_a;
    vec2 ndc_b = clip_b.xy / w_b;

    vec2 screen_a = ndc_a * u_viewport * 0.5;
    vec2 screen_b = ndc_b * u_viewport * 0.5;

    vec2 dir = screen_b - screen_a;
    float len = length(dir);
    vec2 normal = len > 0.0001 ? vec2(-dir.y, dir.x) / len : vec2(0.0, 1.0);

    // Mitad del grosor en píxeles (mínimo 1.2 px para nitidez antialias)
    float half_width = max(u_line_width, 1.2) * 0.5;
    vec2 offset_px = normal * (a_meta.x * half_width);
    vec2 offset_ndc = (offset_px * 2.0) / max(u_viewport, vec2(1.0));

    gl_Position = clip_a;
    gl_Position.xy += offset_ndc * clip_a.w;

    v_edge = a_meta.x;
    v_intensity = a_meta.y;
}
