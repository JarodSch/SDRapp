#include <metal_stdlib>
using namespace metal;

struct SpectrumVertex {
    float4 position [[position]];
    float4 color;
};

vertex SpectrumVertex spectrum_vertex(
    uint vid [[vertex_id]],
    constant float* fftData [[buffer(0)]],
    constant uint& count    [[buffer(1)]]
) {
    float x = float(vid) / float(count - 1) * 2.0 - 1.0;
    float normalized = (fftData[vid] + 120.0) / 120.0;
    float y = normalized * 1.8 - 0.9;
    float4 color = float4(0.784, 0.722, 0.290, 1.0); // amber #c8b84a
    return { float4(x, y, 0.0, 1.0), color };
}

fragment float4 spectrum_fragment(SpectrumVertex in [[stage_in]]) {
    return in.color;
}

vertex SpectrumVertex spectrum_fill_vertex(
    uint vid [[vertex_id]],
    constant float* fftData [[buffer(0)]],
    constant uint& count    [[buffer(1)]]
) {
    uint bin = vid / 2;
    float x = float(bin) / float(count - 1) * 2.0 - 1.0;
    float normalized = (fftData[bin] + 120.0) / 120.0;
    float yTop = normalized * 1.8 - 0.9;
    float y = (vid % 2 == 0) ? yTop : -0.9;
    float alpha = (vid % 2 == 0) ? 0.45 : 0.0;
    return { float4(x, y, 0.0, 1.0), float4(0.784, 0.722, 0.290, alpha) }; // amber
}
