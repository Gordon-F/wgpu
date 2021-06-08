[[block]]
struct Indices {
    arr: array<u32>;
}; // this is used as both input and output for convenience

[[group(0), binding(0)]]
var<storage> indices: [[access(read_write)]] Indices;

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] instance: u32, [[builtin(vertex_index)]] index: u32) -> [[builtin(position)]] vec4<f32> {
    indices.arr[index] = instance;
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
