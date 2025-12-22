#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

// Settings passed from the material
// binding(2) would be the struct if we had one

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, screen_sampler, in.uv);
    
    // 1. Convert to Grayscale (Luminance)
    // Using standard Rec. 709 luma coefficients
    let gray = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

    // 2. Define Palette
    // Paper: Warm, aged parchment (Creamy/Beige)
    let paper_color = vec3<f32>(0.93, 0.88, 0.78); 
    // Ink: Dark, faded iron gall ink (Brownish-Black)
    let ink_color = vec3<f32>(0.15, 0.12, 0.10);   

    // 3. Contrast & Thresholding
    // Increase contrast to make it look more like drawn ink
    // Values closer to 0 become ink, values closer to 1 become paper
    var contrast = 1.2;
    var t = pow(gray, contrast);
    
    // 4. Mix
    let final_rgb = mix(ink_color, paper_color, t);

    // Keep original alpha
    return vec4<f32>(final_rgb, color.a);
}
