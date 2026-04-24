#include <metal_stdlib>
using namespace metal;

// Viridis-ähnliche Colormap (approximiert)
float4 viridis(float t) {
    t = clamp(t, 0.0, 1.0);
    float4 c0 = float4(0.267, 0.005, 0.329, 1.0);
    float4 c1 = float4(0.127, 0.566, 0.551, 1.0);
    float4 c2 = float4(0.993, 0.906, 0.144, 1.0);
    if (t < 0.5) return mix(c0, c1, t * 2.0);
    return mix(c1, c2, (t - 0.5) * 2.0);
}

// Compute: neue FFT-Zeile in Textur schreiben
kernel void waterfall_update(
    texture2d<float, access::write> tex [[texture(0)]],
    constant float* fftData            [[buffer(0)]],
    constant uint&  writeRow           [[buffer(1)]],
    constant uint&  fftCount           [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    if (tid >= fftCount) return;
    float normalized = (fftData[tid] + 120.0) / 120.0;
    tex.write(viridis(normalized), uint2(tid, writeRow));
}

// Vertex: Quad über den gesamten Bildschirm
struct WfVertex {
    float4 pos [[position]];
    float2 uv;
};

vertex WfVertex waterfall_vertex(uint vid [[vertex_id]]) {
    float2 positions[4] = { float2(-1,-1), float2(1,-1), float2(-1,1), float2(1,1) };
    float2 uvs[4]       = { float2(0,1),  float2(1,1),  float2(0,0),  float2(1,0) };
    return { float4(positions[vid], 0, 1), uvs[vid] };
}

// Fragment: sampelt Textur, versetzt um writeRow (aktuelle Zeile oben)
fragment float4 waterfall_fragment(
    WfVertex in             [[stage_in]],
    texture2d<float> tex    [[texture(0)]],
    constant uint& writeRow  [[buffer(0)]],
    constant uint& texHeight [[buffer(1)]]
) {
    constexpr sampler s(filter::nearest);
    float rowOffset = float(writeRow) / float(texHeight);
    float2 uv = float2(in.uv.x, fract(in.uv.y + rowOffset));
    return tex.sample(s, uv);
}
