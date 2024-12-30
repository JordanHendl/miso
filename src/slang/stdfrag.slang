// Fragment Shader (fragment.slang)
struct FSInput
{
    float3 color : COLOR;
};

struct FSOutput
{
    float4 color : SV_TARGET;
};

FSOutput main(FSInput input)
{
    FSOutput output;
    output.color = float4(input.color, 1.0); // Output the interpolated vertex color
    return output;
}

