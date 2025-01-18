// Fragment Shader (fragment.slang)
import bindless;
import miso;

struct FSInput
{
    float3 color : COLOR;
    float2 tex_coords;
};

struct FSOutput
{
    float4 color : SV_TARGET;
};

[shader("fragment")]
FSOutput main(FSInput input)
{
    FSOutput output;
    if(per_obj.material.base_color.valid()) {
      let t = bless_textures[per_obj.material.base_color.id()]; 
      output.color = t.t.Sample(t.s, input.tex_coords); 
    } else {
      output.color = float4(input.color, 1.0); // Output the interpolated vertex color
    }

    return output;
}

