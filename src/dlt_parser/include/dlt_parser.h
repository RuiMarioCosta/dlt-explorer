#pragma once

#include "dlt_parser_export.h"

#include "buffer.h"

#include <boost/interprocess/mapped_region.hpp>

#include <cstdint>
#include <filesystem>
#include <string_view>
#include <vector>

void parse_dlt_explorer(std::filesystem::path const &path);

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

  // friend std::ostream &operator<<(std::ostream &ostream, DLT const &dlt);
};
