// Fragment Shader (fragment.slang)

import miso;

struct FSInput
{
    float3 color : COLOR;
};

struct FSOutput
{
    float4 color : SV_TARGET;
};

[shader("fragment")]
FSOutput main(FSInput input)
{
    FSOutput output;
    output.color = float4(input.color, 1.0); // Output the interpolated vertex colo:
    return output;
}

