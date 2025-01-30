struct Uniforms {
    input_size: vec2<u32>,
    output_size: vec2<u32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> input_data: array<u32>;
@group(0) @binding(2) var<storage, read_write> output_data: array<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.output_size.x || y >= uniforms.output_size.y) {
        return;
    }

    let scale_x = f32(uniforms.input_size.x) / f32(uniforms.output_size.x);
    let scale_y = f32(uniforms.input_size.y) / f32(uniforms.output_size.y);

    // バイリニア補間の座標計算
    let src_x = f32(x) * scale_x;
    let src_y = f32(y) * scale_y;
    
    let x0 = u32(floor(src_x));
    let y0 = u32(floor(src_y));
    let x1 = min(x0 + 1u, uniforms.input_size.x - 1u);
    let y1 = min(y0 + 1u, uniforms.input_size.y - 1u);
    
    let fx = src_x - f32(x0);
    let fy = src_y - f32(y0);

    // 4つの近傍ピクセルを取得
    let c00 = get_pixel(x0, y0);
    let c10 = get_pixel(x1, y0);
    let c01 = get_pixel(x0, y1);
    let c11 = get_pixel(x1, y1);

    // バイリニア補間を実行
    let c0 = mix(c00, c10, fx);
    let c1 = mix(c01, c11, fx);
    let final_color = mix(c0, c1, fy);

    // 結果を書き込み
    let output_index = (y * uniforms.output_size.x + x) * 3u;
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