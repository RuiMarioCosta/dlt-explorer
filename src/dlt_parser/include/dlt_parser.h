#pragma once

#include "dlt_parser_export.h"

#include "buffer.h"

#include <boost/interprocess/mapped_region.hpp>
#include <fmt/chrono.h>
#include <fmt/format.h>

#include <chrono>
#include <cstdint>
#include <filesystem>
#include <span>
#include <string_view>
#include <vector>


class DLT {
  std::filesystem::path m_path;
  boost::interprocess::mapped_region m_region;

  std::vector<std::string_view> m_patterns;
  std::vector<uint32_t> m_seconds;
  std::vector<int32_t> m_microseconds;
  std::vector<std::string_view> m_ecus;
  std::vector<uint8_t> m_htyps;
  std::vector<uint8_t> m_mcnts;
  std::vector<uint16_t> m_lens;
  std::vector<uint32_t> m_seids;
  std::vector<uint32_t> m_tmsps;
  std::vector<uint8_t> m_msins;
  std::vector<uint8_t> m_noars;
  std::vector<std::string_view> m_apids;
  std::vector<std::string_view> m_ctids;
  std::vector<std::string_view> m_message_types;
  std::vector<std::string_view> m_log_infos;
  std::vector<std::string_view> m_service_id_names;
  std::vector<std::string_view> m_return_types;
  std::vector<std::string_view> m_payloads;
  Buffer m_buffer;

  // invariant: sizes of all vectors must be the same
  size_t m_size{0};

public:
  DLT() = delete;
  explicit DLT_PARSER_EXPORT DLT(std::filesystem::path path);

  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> patterns() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<uint32_t const> seconds() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<int32_t const> microseconds() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> ecus() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<uint8_t const> header_types() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<uint8_t const> message_counters() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<uint16_t const> lengths() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<uint32_t const> session_ids() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<uint32_t const> timestamps() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<uint8_t const> message_infos() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<uint8_t const> number_of_arguments() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> app_ids() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> ctx_ids() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> message_types() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> log_infos() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> service_id_names() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> return_types() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> payloads() const;
  [[nodiscard]] DLT_PARSER_EXPORT size_t size() const;

private:
  void _storageHeader(auto iterator);
  void _standardHeader(auto iterator);
};

template<>
struct fmt::formatter<DLT> : formatter<string_view> {
  auto format(DLT const &dlt, format_context &ctx) const {
    fmt::memory_buffer buf;
    for (size_t i = 0; i < dlt.size(); i++) {
      std::chrono::system_clock::duration seconds =
        std::chrono::seconds{dlt.seconds()[i]} + std::chrono::microseconds{dlt.microseconds()[i]};
      std::chrono::time_point<std::chrono::system_clock> tp{seconds};

      fmt::format_to(std::back_inserter(buf),
        "|{}|{:.4f}|{:4}|{:4}|{:4}|{:8}|{:8}|{}|\n",
        tp,
        dlt.timestamps()[i] / 1000.0,// convert from 0.1 ms to sec
        dlt.ecus()[i],
        dlt.app_ids()[i],
        dlt.ctx_ids()[i],
        dlt.message_types()[i],
        dlt.log_infos()[i],
        dlt.payloads()[i]);
    }
    return fmt::format_to(ctx.out(), "{}", to_string(buf));
  }
};
