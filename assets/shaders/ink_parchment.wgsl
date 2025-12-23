#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var paper_texture: texture_2d<f32>;
@group(0) @binding(3) var paper_sampler: sampler;

struct AestheticSettings {
    // Epic 8.3: Paper
    paper_texture_strength: f32,
    vignette_strength: f32,
    vignette_radius: f32,
    grain_strength: f32,
    grain_scale: f32,
    stain_strength: f32,
    ink_feather_radius: f32,
    // Epic 8.4: Edge Detection
    edge_detection_enabled: u32,
    edge_threshold: f32,
    wobble_amplitude: f32,
    wobble_frequency: f32,
    edge_thickness: f32,
    crosshatch_enabled: u32,
    crosshatch_density: f32,
    // Time
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
// Edge Detection Utilities
// ============================================================================

// Convert RGB to grayscale luminance
fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// Sample screen texture and return luminance
fn sample_lum(uv: vec2<f32>) -> f32 {
    return luminance(textureSample(screen_texture, screen_sampler, uv).rgb);
}

// Sobel edge detection - samples 3x3 neighborhood and returns edge magnitude
fn sobel(uv: vec2<f32>, texel: vec2<f32>) -> f32 {
    // Sample 3x3 neighborhood (grayscale luminance)
    let tl = sample_lum(uv + vec2<f32>(-1.0, -1.0) * texel);
    let t  = sample_lum(uv + vec2<f32>( 0.0, -1.0) * texel);
    let tr = sample_lum(uv + vec2<f32>( 1.0, -1.0) * texel);
    let l  = sample_lum(uv + vec2<f32>(-1.0,  0.0) * texel);
    let r  = sample_lum(uv + vec2<f32>( 1.0,  0.0) * texel);
    let bl = sample_lum(uv + vec2<f32>(-1.0,  1.0) * texel);
    let b  = sample_lum(uv + vec2<f32>( 0.0,  1.0) * texel);
    let br = sample_lum(uv + vec2<f32>( 1.0,  1.0) * texel);

    // Sobel kernels for horizontal and vertical gradients
    let gx = -tl - 2.0 * l - bl + tr + 2.0 * r + br;
    let gy = -tl - 2.0 * t - tr + bl + 2.0 * b + br;

    // Return edge magnitude
    return sqrt(gx * gx + gy * gy);
}

// Wobble UV for hand-drawn imperfection - displaces edge detection sampling
fn wobble_uv(uv: vec2<f32>, time: f32, texel: vec2<f32>, amplitude: f32, frequency: f32) -> vec2<f32> {
    // Slow animation (time * 0.5) for organic drift, not distracting jitter
    let anim_time = time * 0.5;
    // Different frequencies for X/Y (1.3x) to avoid uniform, artificial waves
    let wobble_x = sin(uv.y * frequency + anim_time) * amplitude * texel.x;
    let wobble_y = cos(uv.x * frequency * 1.3 + anim_time) * amplitude * texel.y;
    return uv + vec2<f32>(wobble_x, wobble_y);
}

// Crosshatch pattern for shadow shading - returns 0.0 (ink) or 1.0 (paper)
fn crosshatch(uv: vec2<f32>, density: f32) -> f32 {
    let scale = 100.0;
    // Two diagonal line patterns (crossing each other)
    let line1 = abs(sin((uv.x + uv.y) * scale));
    let line2 = abs(sin((uv.x - uv.y) * scale));
    // Second layer slightly denser (0.7x threshold)
    let hatch1 = step(density, line1);
    let hatch2 = step(density * 0.7, line2);
    // Both need to pass for paper to show through
    return min(hatch1, hatch2);
}

// ============================================================================
// Main Fragment Shader
// ============================================================================

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, screen_sampler, in.uv);

    // Get texture dimensions for texel size calculation
    let tex_size = vec2<f32>(textureDimensions(screen_texture));
    let texel = 1.0 / tex_size;

    // 1. Convert to Grayscale (Luminance)
    let gray = luminance(color.rgb);

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
    let paper_gray = luminance(paper.rgb);
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

    // 7. Sobel Edge Detection (Epic 8.4)
    if (settings.edge_detection_enabled != 0u) {
        // Apply wobble to UV for hand-drawn imperfection (8.4.3)
        let wobbled_uv = wobble_uv(in.uv, settings.time, texel, settings.wobble_amplitude, settings.wobble_frequency);
        
        // Initial edge detection for line weight calculation
        let initial_edge = sobel(wobbled_uv, texel);
        
        // Variable line weight (8.4.4): thicker lines for stronger edges
        let line_weight = mix(1.0, settings.edge_thickness, initial_edge);
        let weighted_texel = texel * line_weight;
        
        // Re-sample with dilated kernel for thicker edges
        let edge = max(
            sobel(wobbled_uv, weighted_texel),
            sobel(wobbled_uv + texel * 0.5, weighted_texel)
        );
        
        // Soft threshold for smooth edge transition
        let edge_mask = smoothstep(settings.edge_threshold, settings.edge_threshold + 0.1, edge);
        
        // Mute the fill colors (push toward paper) per docs/weathered.md
        let muted_color = mix(final_rgb, PAPER_COLOR, 0.3);
        
        // Draw edges in ink, keep muted fill elsewhere
        final_rgb = mix(muted_color, INK_COLOR, edge_mask);
    }

    // 8. Crosshatch Shading for Shadows (8.4.5)
    if (settings.crosshatch_enabled != 0u) {
        // Calculate darkness (inverse of luminance)
        let darkness = 1.0 - luminance(final_rgb);
        
        // Only apply crosshatch to very dark areas (threshold at 0.5)
        if (darkness > 0.5) {
            let hatch_mask = crosshatch(in.uv, settings.crosshatch_density);
            let hatched = mix(INK_COLOR, PAPER_COLOR, hatch_mask);
            // Blend subtly based on how dark the area is (darker = more hatching)
            let hatch_strength = smoothstep(0.5, 0.8, darkness);
            final_rgb = mix(final_rgb, hatched, hatch_strength * 0.15);
        }
    }

    // 9. Water Ink Wash Effect (8.5.5)
    // Detects blue-ish water colors and applies watercolor bleeding at edges
    {
        // Water detection: check if current pixel is blue-ish (water tiles)
        let blue_ratio = final_rgb.b / max(final_rgb.r + final_rgb.g + 0.001, 0.1);
        let is_water = step(0.4, blue_ratio) * step(0.2, final_rgb.b);
        
        // Sample neighbors to detect color transitions (potential coastlines)
        let wash_texel = texel * 2.0; // Larger sampling radius for softer bleed
        let neighbor_tl = textureSample(screen_texture, screen_sampler, in.uv + vec2<f32>(-1.0, -1.0) * wash_texel).rgb;
        let neighbor_tr = textureSample(screen_texture, screen_sampler, in.uv + vec2<f32>( 1.0, -1.0) * wash_texel).rgb;
        let neighbor_bl = textureSample(screen_texture, screen_sampler, in.uv + vec2<f32>(-1.0,  1.0) * wash_texel).rgb;
        let neighbor_br = textureSample(screen_texture, screen_sampler, in.uv + vec2<f32>( 1.0,  1.0) * wash_texel).rgb;
        
        // Calculate color variance (high variance = edge/transition)
        let avg_neighbor = (neighbor_tl + neighbor_tr + neighbor_bl + neighbor_br) * 0.25;
        let color_diff = distance(final_rgb, avg_neighbor);
        
        // Only apply wash at transitions (where color differs from neighbors)
        let transition_mask = smoothstep(0.05, 0.2, color_diff);
        
        // Blend toward a soft water-tinted wash at transitions
        // Use a subtle teal/blue wash color for watercolor effect
        let wash_tint = vec3<f32>(0.6, 0.75, 0.8);
        let wash_color = mix(final_rgb, wash_tint, 0.15);
        
        // Apply wash only at water edges (water pixels near transitions)
        let wash_strength = is_water * transition_mask * 0.4;
        final_rgb = mix(final_rgb, wash_color, wash_strength);
    }

    return vec4<f32>(final_rgb, color.a);
}

