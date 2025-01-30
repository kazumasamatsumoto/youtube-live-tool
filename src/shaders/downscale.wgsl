struct Uniforms {
    input_size: vec2<u32>,
    output_size: vec2<u32>,
    scale: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> input_data: array<u32>;
@group(0) @binding(2) var<storage, read_write> output_data: array<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_pos = vec2<u32>(global_id.xy);
    let input_pos = vec2<f32>(output_pos) * uniforms.scale;
    
    // バイリニア補間による縮小処理
    let x0 = floor(input_pos.x);
    let y0 = floor(input_pos.y);
    let x1 = min(x0 + 1.0, f32(uniforms.input_size.x - 1));
    let y1 = min(y0 + 1.0, f32(uniforms.input_size.y - 1));
    
    let sx = input_pos.x - x0;
    let sy = input_pos.y - y0;
    
    // 4つの近傍ピクセルを取得
    let c00 = get_pixel(u32(x0), u32(y0));
    let c10 = get_pixel(u32(x1), u32(y0));
    let c01 = get_pixel(u32(x0), u32(y1));
    let c11 = get_pixel(u32(x1), u32(y1));

    // バイリニア補間を実行
    let c0 = mix(c00, c10, sx);
    let c1 = mix(c01, c11, sx);
    let final_color = mix(c0, c1, sy);

    // 結果を書き込み
    let output_index = (output_pos.y * uniforms.output_size.x + output_pos.x) * 3u;
    output_data[output_index] = u32(final_color.r * 255.0);
    output_data[output_index + 1u] = u32(final_color.g * 255.0);
    output_data[output_index + 2u] = u32(final_color.b * 255.0);
}

fn get_pixel(x: u32, y: u32) -> vec4<f32> {
    let index = (y * uniforms.input_size.x + x) * 3u;
    return vec4<f32>(
        f32(input_data[index]) / 255.0,
        f32(input_data[index + 1u]) / 255.0,
        f32(input_data[index + 2u]) / 255.0,
        1.0
    );
} 