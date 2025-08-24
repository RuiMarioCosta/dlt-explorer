#pragma once

#include "buffer_export.h"

#include <fmt/base.h>
#include <fmt/format.h>
#include <fmt/ranges.h>

#include <concepts>
#include <ranges>
#include <span>
#include <string_view>
#include <utility>
#include <vector>


template<typename T>
struct Hex {
  T value;
};

template<std::integral T>
struct fmt::formatter<Hex<T>> : formatter<string_view> {
  auto format(Hex<T> hex, format_context &ctx) const { return fmt::format_to(ctx.out(), "{:#x}", hex.value); }
};

template<>
struct fmt::formatter<Hex<std::string_view>> : formatter<string_view> {
  auto format(Hex<std::string_view> hex, format_context &ctx) const {
    return fmt::format_to(ctx.out(), "{:02x}", fmt::join(hex.value, " "));
  }
};


template<typename T>
struct Bin {
  T value;
};

template<std::integral T>
struct fmt::formatter<Bin<T>> : formatter<string_view> {
  auto format(Bin<T> binary, format_context &ctx) const { return fmt::format_to(ctx.out(), "{:#b}", binary.value); }
};


class Buffer {
  std::vector<char> m_buffer;

public:
  BUFFER_EXPORT Buffer() = default;
  explicit BUFFER_EXPORT Buffer(size_t capacity);

  [[nodiscard]] BUFFER_EXPORT size_t size() const;
  [[nodiscard]] BUFFER_EXPORT size_t capacity() const;

  template<typename T>
    requires(not std::ranges::range<T>)
  std::string_view store(T &&args) {
    auto old_size = m_buffer.size();
    fmt::format_to(std::back_inserter(m_buffer), "{}", std::forward<T>(args));
    return {m_buffer.data() + old_size, m_buffer.size() - old_size};
  }

  template<std::ranges::range T>
  std::string_view store(T &&args) {
    auto old_size = m_buffer.size();
    fmt::format_to(std::back_inserter(m_buffer), "{}", fmt::join(std::forward<T>(args), " "));
    return {m_buffer.data() + old_size, m_buffer.size() - old_size};
  }

  std::string_view store(auto &&...args) {
    auto old_size = m_buffer.size();
    std::array<std::string, sizeof...(args)> parts{fmt::format("{}", std::forward<decltype(args)>(args))...};
    fmt::format_to(std::back_inserter(m_buffer), "{}", fmt::join(parts, " "));
    return {m_buffer.data() + old_size, m_buffer.size() - old_size};
  }
};
