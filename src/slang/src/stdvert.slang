// Vertex Shader (vertex.slang)
import bindless;
import miso;

struct VSInput
{
    float4 position;
    float4 normal;
    float2 tex_coords;
    int4 joint_ids;
    float4 joints;
};

struct VSOutput
{
    float4 position : SV_POSITION;
    float3 color : COLOR;
    float2 tex_coords;
}

[shader("vertex")]
VSOutput main(VSInput input)
{
    VSOutput output;
    output.position = mul(per_obj.transform, float4(input.position.xyz, 1.0));
    output.color = input.position.xyz;
    output.tex_coords = input.position.xy;
    return output;
}

