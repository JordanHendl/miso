// Vertex Shader (vertex.slang)
import bindless;
import miso;

struct VSInput
{
    float4 position;
    float4 normal;
    float4 color;
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
    float4x4 t = transpose(per_obj.transform);
    output.position = mul(float4(input.position.xyz, 1.0), per_obj.transform);
    output.color = input.color.xyz;
    output.tex_coords = input.tex_coords.xy;
    return output;
}

