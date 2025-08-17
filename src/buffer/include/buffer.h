#pragma once

#include "buffer_export.h"

#include <fmt/base.h>

#include <string_view>
#include <vector>

struct Hex
{
  long value;
};

template<> struct fmt::formatter<Hex> : formatter<string_view>
{
  auto format(Hex hex, format_context &ctx) const { return fmt::format_to(ctx.out(), "0x{:x}", hex.value); }
};


struct Bin
{
  long value;
};

template<> struct fmt::formatter<Bin> : formatter<string_view>
{
  auto format(Bin binary, format_context &ctx) const { return fmt::format_to(ctx.out(), "0b{:b}", binary.value); }
};


class Buffer
{
  std::vector<char> m_buffer;

public:
  explicit BUFFER_EXPORT Buffer(size_t capacity);

  [[nodiscard]] BUFFER_EXPORT size_t size() const;
  [[nodiscard]] BUFFER_EXPORT size_t capacity() const;

  std::string_view store(auto args)
  {
    auto old_size = m_buffer.size();
    fmt::format_to(std::back_inserter(m_buffer), "{}", args);
    return { m_buffer.data() + old_size, m_buffer.size() - old_size };
  }
};
