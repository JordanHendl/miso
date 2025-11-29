struct Handle {
  uint packed;

  bool valid() {
    uint id = packed & 0xFFFF;
    uint gen = packed >> 16;
    return id != 65535 && gen != 65535;
  }

  uint id() {
    return packed & 0xFFFF;
  }

  uint generation() {
    return packed >> 16;
  }
};
