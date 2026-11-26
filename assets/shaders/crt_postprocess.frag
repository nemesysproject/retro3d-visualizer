in vec2 v_uv;

uniform sampler2D u_scene;
uniform vec2 u_resolution;
uniform vec3 u_phosphor_color;
uniform float u_crt_enabled;
uniform float u_glow_intensity;
uniform float u_time;

out vec4 out_color;

vec2 barrel_distort(vec2 uv) {
    vec2 c = uv * 2.0 - 1.0;
    float r2 = dot(c, c);
    c *= 1.0 + 0.085 * r2;
    return c * 0.5 + 0.5;
}

vec3 sample_bloom(vec2 uv, vec2 texel) {
    vec3 acc = vec3(0.0);
    // Muestreo en cruz y diagonales a 2 radios para resplandor de fósforo
    vec2 offsets[8];
    offsets[0] = vec2( 1.0,  0.0);
    offsets[1] = vec2(-1.0,  0.0);
    offsets[2] = vec2( 0.0,  1.0);
    offsets[3] = vec2( 0.0, -1.0);
    offsets[4] = vec2( 0.707,  0.707);
    offsets[5] = vec2(-0.707,  0.707);
    offsets[6] = vec2( 0.707, -0.707);
    offsets[7] = vec2(-0.707, -0.707);

    for (int i = 0; i < 8; i++) {
        acc += texture(u_scene, uv + offsets[i] * texel * 2.2).rgb * 0.09;
        acc += texture(u_scene, uv + offsets[i] * texel * 5.0).rgb * 0.055;
    }
    return acc;
}

void main() {
    vec2 uv = v_uv;

    if (u_crt_enabled > 0.5) {
        uv = barrel_distort(uv);
        if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
            out_color = vec4(0.01, 0.015, 0.012, 1.0);
            return;
        }
    }

    vec2 texel = 1.0 / max(u_resolution, vec2(1.0));
    vec3 base = texture(u_scene, uv).rgb;
    vec3 bloom = sample_bloom(uv, texel) * u_glow_intensity;

    vec3 color = base + bloom;

    if (u_crt_enabled > 0.5) {
        // Ligera separación cromática horizontal de tubo CRT
        float shift = texel.x * 1.1;
        float r_ch = texture(u_scene, uv + vec2(shift, 0.0)).r;
        float b_ch = texture(u_scene, uv - vec2(shift, 0.0)).b;
        color = mix(color, vec3(r_ch, color.g, b_ch) + bloom, 0.35);

        // Scanlines horizontales retro
        float scan = sin(uv.y * u_resolution.y * 3.14159265) * 0.5 + 0.5;
        float scan_mask = mix(0.72, 1.0, scan);
        color *= scan_mask;

        // Sutil parpadeo de barrido vertical
        float flicker = 0.985 + 0.015 * sin(u_time * 42.0);
        color *= flicker;

        // Fondo de tubo de rayos catódicos con leve brillo de fósforo
        vec2 centered = uv * 2.0 - 1.0;
        float vignette = clamp(1.0 - dot(centered, centered) * 0.28, 0.25, 1.0);
        vec3 crt_ambient = u_phosphor_color * 0.022 * vignette;
        color = (color + crt_ambient) * vignette;
    }

    // Tone mapping suave para evitar saturación dura
    color = 1.0 - exp(-color * 1.35);
    out_color = vec4(color, 1.0);
}
