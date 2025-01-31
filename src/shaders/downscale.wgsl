struct Uniforms {
    input_size: vec2<u32>,
    output_size: vec2<u32>,
    scale: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> input_data: array<u32>;
@group(0) @binding(2) var<storage, read_write> output_data: array<u32>;

// ワークグループサイズを16x16に変更して制限内に収める
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_pos = vec2<u32>(global_id.xy);
    
    // 出力範囲チェックを追加して不要な計算を回避
    if (output_pos.x >= uniforms.output_size.x || output_pos.y >= uniforms.output_size.y) {
        return;
    }

    let input_pos = vec2<f32>(output_pos) * uniforms.scale;
    
    // 整数部分と小数部分を効率的に分離
    let x0 = u32(input_pos.x);
    let y0 = u32(input_pos.y);
    let x1 = min(x0 + 1u, uniforms.input_size.x - 1u);
    let y1 = min(y0 + 1u, uniforms.input_size.y - 1u);
    
    let sx = input_pos.x - f32(x0);
    let sy = input_pos.y - f32(y0);

    // 一度に4ピクセル分のデータを読み込み
    let c00 = get_pixel_fast(x0, y0);
    let c10 = get_pixel_fast(x1, y0);
    let c01 = get_pixel_fast(x0, y1);
    let c11 = get_pixel_fast(x1, y1);

    // 線形補間を最適化（乗算回数を削減）
    let inv_sx = 1.0 - sx;
    let inv_sy = 1.0 - sy;
    
    let top = c00 * inv_sx + c10 * sx;
    let bottom = c01 * inv_sx + c11 * sx;
    let final_color = top * inv_sy + bottom * sy;

    // 出力インデックスの計算を最適化
    let output_index = (output_pos.y * uniforms.output_size.x + output_pos.x) * 3u;
    
    // 一度に3要素を書き込み
    output_data[output_index] = u32(final_color.x * 255.0 + 0.5);
    output_data[output_index + 1u] = u32(final_color.y * 255.0 + 0.5);
    output_data[output_index + 2u] = u32(final_color.z * 255.0 + 0.5);
}

// 最適化されたピクセル取得関数
fn get_pixel_fast(x: u32, y: u32) -> vec3<f32> {
    let index = (y * uniforms.input_size.x + x) * 3u;
    return vec3<f32>(
        f32(input_data[index]),
        f32(input_data[index + 1u]),
        f32(input_data[index + 2u])
    ) / 255.0;
}
