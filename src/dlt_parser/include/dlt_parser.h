#pragma once

#include "dlt_parser_export.h"

#include "buffer.h"

#include <boost/interprocess/mapped_region.hpp>

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
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> service_id_names() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> return_types() const;
  [[nodiscard]] DLT_PARSER_EXPORT std::span<std::string_view const> payloads() const;
  [[nodiscard]] DLT_PARSER_EXPORT size_t size() const;

  // friend std::ostream &operator<<(std::ostream &ostream, DLT const &dlt);
private:
  // all types used in serialization
  using all_t =
    std::variant<std::string_view, int8_t, int16_t, int32_t, int64_t, uint8_t, uint16_t, uint32_t, uint64_t>;

  void _storageHeader(auto iterator);
  void _standardHeader(auto iterator);
};
