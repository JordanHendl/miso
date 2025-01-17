// Vertex Shader (vertex.slang)
import bindless;
import miso;

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
    output.position = mul(per_obj.transform, float4(input.position.xyz, 1.0));
    output.color = input.color;
    return output;
}

