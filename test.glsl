#version 450
layout(row_major) uniform;
layout(row_major) buffer;

#line 2 0
struct PerFrameInfo_std140_0
{
    vec4 camera_position_0;
};


#line 26
layout(binding = 1)
layout(std140) uniform _S1
{
    vec4 camera_position_0;
}per_frame_0;

#line 4 1
layout(location = 0)
out vec3 entryPointParam_main_color_0;


#line 4
layout(location = 1)
in vec3 input_color_0;




struct VSOutput_0
{
    vec4 position_0;
    vec3 color_0;
};


void main()
{
    VSOutput_0 output_0;
    output_0.color_0 = input_color_0 + per_frame_0.camera_position_0.xyz;
    VSOutput_0 _S2 = output_0;

#line 21
    gl_Position = output_0.position_0;

#line 21
    entryPointParam_main_color_0 = _S2.color_0;

#line 21
    return;
}

