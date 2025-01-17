namespace miso {
struct Handle {
  uint16_t m_id;
  uint16_t m_gen;

  bool valid() {
    return m_id != 65535;
  }

  uint16_t id() {
    return m_id;
  }
}
}
