#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var paper_texture: texture_2d<f32>;
@group(0) @binding(3) var paper_sampler: sampler;

struct AestheticSettings {
    paper_texture_strength: f32,
    vignette_strength: f32,
    vignette_radius: f32,
    grain_strength: f32,
    grain_scale: f32,
    stain_strength: f32,
    ink_feather_radius: f32,
    time: f32,
}

@group(0) @binding(4) var<uniform> settings: AestheticSettings;

// Paper: Warm, aged parchment (Creamy/Beige)
const PAPER_COLOR: vec3<f32> = vec3<f32>(0.93, 0.88, 0.78);
// Ink: Dark, faded iron gall ink (Brownish-Black)
const INK_COLOR: vec3<f32> = vec3<f32>(0.15, 0.12, 0.10);

// ============================================================================
// Noise Utility Functions for Paper Grain
// ============================================================================

// Pseudo-random hash from 2D input -> scalar output
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + dot(p3, vec3<f32>(p3.y + 33.33, p3.z + 33.33, p3.x + 33.33));
    return fract((p3.x + p3.y) * p3.z);
}

// Value noise (smooth random) - interpolates between random values at grid points
fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    // Cubic Hermite interpolation (smoother than linear)
    let u = f * f * (3.0 - 2.0 * f);
    
    // Sample grid corners
    let a = hash21(i + vec2<f32>(0.0, 0.0));
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    
    // Bilinear interpolation
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Fractal Brownian Motion - layered noise at multiple octaves
fn fbm(uv: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    
    // 4 octaves balances quality vs performance
    for (var i = 0; i < 4; i = i + 1) {
        value = value + amplitude * noise2d(uv * frequency);
        frequency = frequency * 2.0;
        amplitude = amplitude * 0.5;
    }
    
    return value;
}

// ============================================================================
// Main Fragment Shader
// ============================================================================

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, screen_sampler, in.uv);

    // 1. Convert to Grayscale (Luminance)
    let gray = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

    // 2. Contrast & Thresholding
    var contrast = 1.2;
    var t = pow(gray, contrast);

    // 3. Base ink/paper mix
    var final_rgb = mix(INK_COLOR, PAPER_COLOR, t);

    // 4. Paper Texture Overlay
    // Tile the paper texture across the screen (3x scale to avoid obvious tiling)
    let paper_tile_scale = 3.0;
    // Add slight rotation to break grid alignment
    let angle = 0.02;
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    let rotated_uv = vec2<f32>(
        in.uv.x * cos_a - in.uv.y * sin_a,
        in.uv.x * sin_a + in.uv.y * cos_a
    );
    let paper_uv = rotated_uv * paper_tile_scale;
    let paper = textureSample(paper_texture, paper_sampler, paper_uv);

    // Paper texture modulates the final color
    // We use the paper's luminance deviation from 0.5 to add texture
    let paper_gray = dot(paper.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let paper_factor = (paper_gray - 0.5) * settings.paper_texture_strength;

    // Apply paper texture more to lighter areas (paper shows through more on light parts)
    let paper_mask = t; // Lighter areas get more texture
    final_rgb = final_rgb + paper_factor * paper_mask;

    // 5. Vignette Effect
    // Radial darkening from center, shaped like natural light falloff on aged paper
    let center = vec2<f32>(0.5, 0.5);
    // Slightly asymmetric: heavier at bottom to mimic aged document
    let adjusted_uv = vec2<f32>(in.uv.x, in.uv.y * 1.1 - 0.05);
    let dist = distance(adjusted_uv, center);
    // Squared distance for natural falloff
    let vignette = smoothstep(settings.vignette_radius, settings.vignette_radius + 0.35, dist);
    // Darken toward ink color, not black (maintains palette)
    final_rgb = mix(final_rgb, INK_COLOR, vignette * settings.vignette_strength);

    // 6. Paper Grain (FBM Noise)
    // Slow animation (0.001 * time) for organic feel without distraction
    let grain_uv = in.uv * settings.grain_scale + settings.time * 0.001;
    let grain = fbm(grain_uv);
    let grain_adjust = (grain - 0.5) * settings.grain_strength;
    // Modulate grain by luminance - more visible on lighter paper areas
    final_rgb = final_rgb + grain_adjust * t;

    return vec4<f32>(final_rgb, color.a);
}
