  struct DirectionalLight {
    float4 direction;
    float4 color;
    float intensity;
    float3 pad0;
  };

  struct OptionalDirectionalLight {
    DirectionalLight m_item;
    uint m_present;
    uint3 m_padding;

    bool valid() {
      return m_present != 0;
    }

    DirectionalLight get() {
      return m_item;
    }
  };

  struct Light {
    OptionalDirectionalLight dir_light;
  };
