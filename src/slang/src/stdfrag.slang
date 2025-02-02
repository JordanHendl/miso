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

bool valid_color(float4 color) {
  return color.x != 0.0 && color.y != 0.0 && color.z != 0.0 && color.w != 0.0;
}

[shader("fragment")]
FSOutput main(FSInput input)
{
    FSOutput output;
    if(per_obj.material.base_color.valid()) {
      let t = bless_textures[per_obj.material.base_color.id()]; 
      output.color = t.t.Sample(t.s, input.tex_coords); 
    } else if(valid_color(per_obj.material.base_color_factor)) {
      output.color = float4(per_obj.material.base_color_factor);
    } else {
      output.color = float4(input.color, 1.0); // Output the interpolated vertex color
    }

    if(per_obj.material.emissive.valid()) {
      let t = bless_textures[per_obj.material.emissive.id()]; 
      output.color *= t.t.Sample(t.s, input.tex_coords); 
    } else if(valid_color(per_obj.material.emissive_factor)) {
      output.color *= float4(per_obj.material.emissive_factor);
    }

    return output;
}

