// Vertex Shader (vertex.slang)
[[vk::binding(0)]]
cbuffer Transform : register(b0)
{
    float4x4 modelViewProjection;
};

struct VSInput
{
    float3 position : POSITION;
    float3 color : COLOR;
};

struct VSOutput
{
    float4 position : SV_POSITION;
    float3 color : COLOR;
};

[shader("vertex")]
VSOutput main(VSInput input)
{
    VSOutput output;
    output.position = mul(modelViewProjection, float4(input.position, 1.0));
    output.color = input.color;
    return output;
}

