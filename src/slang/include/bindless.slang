import handle;

struct Material {
  miso::Handle base_color;
  miso::Handle normal;
}

// Bindless textures (all available textures).
struct TextureSamplerPair {
    Texture2D t;
    SamplerState s;
};

// Global settings (environment, rendering settings, etc.).
struct GlobalInfo {
  float app_time_ms;
};

// Per-frame data (camera, transformations, etc.).
struct PerFrameInfo {
  float4 camera_position;
};

// Dynamic per-frame data.
struct PerObjectInfo  {
    float4x4 transform;          // Model matrix.
    Material material;
    float frameIndex;      // Current frame index for animations.
};



[[vk::binding(10)]] uniform TextureSamplerPair bless_textures[];
[[vk::binding(11)]] uniform ConstantBuffer<PerObjectInfo> per_obj;

//uniform ConstantBuffer<GlobalInfo> global;
//uniform ConstantBuffer<PerFrameInfo> per_frame;
