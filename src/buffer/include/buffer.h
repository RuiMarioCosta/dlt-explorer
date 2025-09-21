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
  using iterator = std::vector<char>::iterator;
  using const_iterator = std::vector<char>::const_iterator;


  BUFFER_EXPORT Buffer() = default;
  explicit BUFFER_EXPORT Buffer(size_t capacity);

  [[nodiscard]] BUFFER_EXPORT const char *data_back() const;

  [[nodiscard]] BUFFER_EXPORT iterator begin();
  [[nodiscard]] BUFFER_EXPORT const_iterator cbegin() const;

  [[nodiscard]] BUFFER_EXPORT size_t size() const;
  [[nodiscard]] BUFFER_EXPORT size_t capacity() const;

  template<typename T>
  size_t store(T &&args) {
    const auto *data = data_back();
    fmt::format_to(std::back_inserter(m_buffer), "{}", std::forward<T>(args));
    return static_cast<size_t>(data_back() - data);
  }

  size_t store(auto &&...args) {
    const auto *data = data_back();
    std::array<std::string, sizeof...(args)> parts{fmt::format("{}", std::forward<decltype(args)>(args))...};
    fmt::format_to(std::back_inserter(m_buffer), "{}", fmt::join(parts, " "));
    return static_cast<size_t>(data_back() - data);
  }

private:
  [[nodiscard]] BUFFER_EXPORT iterator _iterator();
  [[nodiscard]] BUFFER_EXPORT const_iterator _citerator() const;
};
