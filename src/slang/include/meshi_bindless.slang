import handle;

struct MeshiCamera {
  float3 position;
  float padding0;
  float4 rotation;
};

struct MeshiTransformation {
  float4x4 transform;
};

struct MeshiTextureInfo {
  uint id;
  uint width;
  uint height;
  uint mip_levels;
};

struct MeshiMaterialInfo {
  uint base_color_texture_id;
  uint normal_texture_id;
  uint metallic_roughness_texture_id;
  uint occlusion_texture_id;
  uint emissive_texture_id;
  uint _padding;
};

struct MeshiTimingInfo {
  float current_time_ms;
  float frame_time_ms;
};

[[vk::binding(0)]] cbuffer meshi_timing
{
  MeshiTimingInfo meshi_timing_data;
};

[[vk::binding(1)]] StructuredBuffer<MeshiCamera> meshi_bindless_camera;
[[vk::binding(2)]] StructuredBuffer<MeshiTextureInfo> meshi_bindless_textures;
[[vk::binding(3)]] StructuredBuffer<MeshiTransformation> meshi_bindless_transformations;
[[vk::binding(4)]] StructuredBuffer<MeshiMaterialInfo> meshi_bindless_materials;

MeshiCamera meshi_default_camera() {
  MeshiCamera cam;
  cam.position = float3(0.0, 0.0, 0.0);
  cam.padding0 = 0.0;
  cam.rotation = float4(0.0, 0.0, 0.0, 1.0);
  return cam;
}

MeshiTransformation meshi_default_transform() {
  MeshiTransformation t;
  t.transform = float4x4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0);
  return t;
}

MeshiTextureInfo meshi_default_texture() {
  MeshiTextureInfo tex;
  tex.id = 0;
  tex.width = 0;
  tex.height = 0;
  tex.mip_levels = 0;
  return tex;
}

MeshiMaterialInfo meshi_default_material() {
  MeshiMaterialInfo mat;
  mat.base_color_texture_id = 0;
  mat.normal_texture_id = 0;
  mat.metallic_roughness_texture_id = 0;
  mat.occlusion_texture_id = 0;
  mat.emissive_texture_id = 0;
  mat._padding = 0;
  return mat;
}

uint meshi_handle_index(Handle h) {
  return (uint)h.id();
}

MeshiCamera meshi_read_camera(Handle h) {
  if(!h.valid()) { return meshi_default_camera(); }
  return meshi_bindless_camera[meshi_handle_index(h)];
}

MeshiTransformation meshi_read_transform(Handle h) {
  if(!h.valid()) { return meshi_default_transform(); }
  return meshi_bindless_transformations[meshi_handle_index(h)];
}

MeshiTextureInfo meshi_read_texture_info(Handle h) {
  if(!h.valid()) { return meshi_default_texture(); }
  return meshi_bindless_textures[meshi_handle_index(h)];
}

MeshiMaterialInfo meshi_read_material(Handle h) {
  if(!h.valid()) { return meshi_default_material(); }
  return meshi_bindless_materials[meshi_handle_index(h)];
}

float meshi_current_time_ms() { return meshi_timing_data.current_time_ms; }
float meshi_frame_time_ms() { return meshi_timing_data.frame_time_ms; }
