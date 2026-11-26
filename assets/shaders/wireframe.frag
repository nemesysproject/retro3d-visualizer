in float v_edge;
in float v_intensity;

uniform vec3 u_color;
uniform float u_glow;

out vec4 out_color;

void main() {
    float dist = abs(v_edge);
    // Núcleo brillante + borde suave tipo haz de electrones en pantalla vectorial
    float core = smoothstep(1.0, 0.15, dist);
    float hot_center = exp(-dist * dist * 4.5);

    vec3 base_col = u_color * (0.45 + 0.95 * v_intensity);
    vec3 beam = base_col * core + vec3(1.0) * (hot_center * 0.28 * v_intensity * u_glow);
    float alpha = clamp(core * (0.35 + 0.65 * v_intensity), 0.0, 1.0);

    out_color = vec4(beam * (0.85 + 0.45 * u_glow), alpha);
}
