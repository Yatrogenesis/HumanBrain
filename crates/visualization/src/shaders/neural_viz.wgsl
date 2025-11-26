// Neural Visualizer - WGSL Shader
// Real-time rendering of multi-compartmental neural dynamics
// Voltage-based color mapping with physically-inspired aesthetics

// Uniform bindings
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> constants: VisConstants;

// Structs
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec3<f32>,
    _padding: f32,
}

struct VisConstants {
    v_rest: f32,        // -70 mV
    v_threshold: f32,   // -55 mV
    v_peak: f32,        // +40 mV
    time: f32,          // Simulation time (ms)
    show_trails: u32,   // Boolean
    trail_length: u32,  // Time steps
    _padding0: u32,
    _padding1: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) voltage: f32,
    @location(2) radius: f32,
    @location(3) comp_type: u32,
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) voltage: f32,
    @location(2) radius: f32,
    @location(3) comp_type: u32,
    @location(4) uv: vec2<f32>,
}

// Voltage to color mapping - Physically inspired
// Blue (hyperpolarized) -> Green (rest) -> Yellow (threshold) -> Red (spike)
fn voltage_to_color(v: f32) -> vec3<f32> {
    // Normalize voltage to [0, 1]
    let v_min = -90.0;  // Hyperpolarized
    let v_max = 50.0;   // Peak spike
    let t = clamp((v - v_min) / (v_max - v_min), 0.0, 1.0);

    // Multi-stage color gradient with smooth transitions
    var color: vec3<f32>;

    if (t < 0.25) {
        // Deep hyperpolarization: Navy blue -> Cyan
        let local_t = t / 0.25;
        color = mix(vec3<f32>(0.0, 0.0, 0.3), vec3<f32>(0.0, 0.5, 0.8), local_t);
    } else if (t < 0.5) {
        // Approaching rest: Cyan -> Green
        let local_t = (t - 0.25) / 0.25;
        color = mix(vec3<f32>(0.0, 0.5, 0.8), vec3<f32>(0.0, 0.7, 0.2), local_t);
    } else if (t < 0.75) {
        // Above rest: Green -> Yellow (threshold crossing)
        let local_t = (t - 0.5) / 0.25;
        color = mix(vec3<f32>(0.0, 0.7, 0.2), vec3<f32>(1.0, 0.9, 0.0), local_t);
    } else {
        // Action potential: Yellow -> Bright Red
        let local_t = (t - 0.75) / 0.25;
        color = mix(vec3<f32>(1.0, 0.9, 0.0), vec3<f32>(1.0, 0.1, 0.0), local_t);
    }

    // Add subtle glow for active compartments
    if (v > constants.v_threshold) {
        let glow_intensity = (v - constants.v_threshold) / (constants.v_peak - constants.v_threshold);
        color = color + vec3<f32>(0.2, 0.2, 0.2) * glow_intensity;
    }

    return color;
}

// Compute billboard corners in view space
fn get_billboard_corner(center: vec3<f32>, corner_id: u32, radius: f32) -> vec3<f32> {
    // Camera right and up vectors (from view matrix)
    let view_right = vec3<f32>(camera.view_proj[0][0], camera.view_proj[1][0], camera.view_proj[2][0]);
    let view_up = vec3<f32>(camera.view_proj[0][1], camera.view_proj[1][1], camera.view_proj[2][1]);

    var offset: vec2<f32>;
    switch corner_id % 4u {
        case 0u: { offset = vec2<f32>(-1.0, -1.0); }
        case 1u: { offset = vec2<f32>(1.0, -1.0); }
        case 2u: { offset = vec2<f32>(1.0, 1.0); }
        default: { offset = vec2<f32>(-1.0, 1.0); }
    }

    // Scale by compartment radius
    let scaled_offset = offset * radius * 0.5;

    return center + view_right * scaled_offset.x + view_up * scaled_offset.y;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Billboard corner ID from vertex index
    let corner_id = input.vertex_index % 4u;

    // Compute billboarded position
    let billboard_pos = get_billboard_corner(input.position, corner_id, input.radius);

    // Transform to clip space
    output.clip_position = camera.view_proj * vec4<f32>(billboard_pos, 1.0);
    output.world_position = billboard_pos;

    // Pass through attributes
    output.voltage = input.voltage;
    output.radius = input.radius;
    output.comp_type = input.comp_type;

    // UV coordinates for corner
    switch corner_id {
        case 0u: { output.uv = vec2<f32>(0.0, 0.0); }
        case 1u: { output.uv = vec2<f32>(1.0, 0.0); }
        case 2u: { output.uv = vec2<f32>(1.0, 1.0); }
        default: { output.uv = vec2<f32>(0.0, 1.0); }
    }

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Circular shape with soft edges
    let uv_centered = input.uv * 2.0 - 1.0;  // [-1, 1]
    let dist = length(uv_centered);

    // Discard pixels outside circle
    if (dist > 1.0) {
        discard;
    }

    // Soft falloff at edges
    let edge_softness = 0.1;
    let alpha = smoothstep(1.0, 1.0 - edge_softness, dist);

    // Get voltage color
    var color = voltage_to_color(input.voltage);

    // Compartment type modulation
    switch input.comp_type {
        case 0u: {  // Soma - brighter, larger
            color = color * 1.3;
        }
        case 1u: {  // Apical dendrite - slight blue tint
            color = color * vec3<f32>(0.9, 0.95, 1.0);
        }
        case 2u: {  // Basal dendrite - slight red tint
            color = color * vec3<f32>(1.0, 0.95, 0.9);
        }
        case 3u: {  // AIS - bright white core
            color = mix(color, vec3<f32>(1.0, 1.0, 1.0), 0.3);
        }
        default: {}
    }

    // Radial gradient for 3D sphere illusion
    let sphere_shading = 1.0 - dist * 0.3;
    color = color * sphere_shading;

    // Subtle ambient occlusion
    let ao = 0.7 + 0.3 * (1.0 - dist);
    color = color * ao;

    // Add specular highlight for spiking compartments
    if (input.voltage > constants.v_threshold) {
        let highlight_center = vec2<f32>(0.3, 0.3);  // Offset highlight
        let highlight_dist = length(uv_centered - highlight_center);
        let specular = exp(-highlight_dist * 8.0) * 0.5;
        color = color + vec3<f32>(specular);
    }

    // Final alpha with edge softness
    let final_alpha = alpha * 0.85;  // Slightly transparent for depth

    return vec4<f32>(color, final_alpha);
}
