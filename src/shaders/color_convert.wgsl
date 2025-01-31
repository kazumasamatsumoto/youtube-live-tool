@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    let coords = vec2<u32>(global_id.xy);
    
    if (coords.x >= dims.x || coords.y >= dims.y) {
        return;
    }

    let color = textureLoad(input_texture, vec2<i32>(coords), 0);
    var converted = color;
    // 赤と青のチャンネルを入れ替え
    converted.r = color.b;
    converted.b = color.r;
    
    textureStore(output_texture, vec2<i32>(coords), converted);
}
