#include "dlt_parser.h"

#include "buffer.h"

/*
 * There are 2 versions of the following 4 headers: one that uses the external dependency, dlt-daemon, and another that
 * is a copy of dlt-daemon but with only the minimum and adapted to C++. The former only works in linux OS therefore the
 * later versions was created to allow portability across operating systems.
 */
#include "adapter.h"
#include "dlt_common.h"
#include "dlt_protocol.h"
#include "dlt_types.h"

#include <boost/interprocess/detail/os_file_functions.hpp>
#include <boost/interprocess/file_mapping.hpp>
#include <boost/interprocess/mapped_region.hpp>
#include <fmt/base.h>

#include <bit>
#include <cassert>
// #include <chrono>
#include <concepts>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <iterator>
// #include <iomanip>
// #include <iostream>
#include <span>
#include <stdexcept>
#include <string_view>
#include <utility>
// #include <sstream>
// #include <string_view>
#include <syslog.h>
// #include <vector>


using namespace boost::interprocess;

namespace {

[[nodiscard]] bool _dlt_msg_is_nonverbose(uint32_t htyp, uint32_t msin) {
  return (!DLT_IS_HTYP_UEH(htyp) || (DLT_IS_HTYP_UEH(htyp) && !DLT_IS_MSIN_VERB(msin)));
}

bool _dlt_msg_is_control(uint32_t htyp, uint32_t msin) {
  return DLT_IS_HTYP_UEH(htyp) && (DLT_GET_MSIN_MSTP(msin) == DLT_TYPE_CONTROL);
}

bool _dlt_msg_is_control_response(uint32_t htyp, uint32_t msin) {
  return DLT_IS_HTYP_UEH(htyp) && (DLT_GET_MSIN_MSTP(msin) == DLT_TYPE_CONTROL)
         && (DLT_GET_MSIN_MTIN(msin) == DLT_CONTROL_RESPONSE);
}

template<typename T>
  requires std::integral<T> || std::floating_point<T>
T dlt_msg_read_value(std::string_view &payload) {
  if (payload.size() < sizeof(T)) { throw std::runtime_error("Payload size is less than expected"); }

  T value;
  std::memcpy(&value, payload.data(), sizeof(value));
  payload.remove_prefix(sizeof(T));
  return value;
}

}// namespace


// NOLINTNEXTLINE(readability-function-cognitive-complexity)
DLT::DLT(std::filesystem::path path) : m_path{std::move(path)} {
  fmt::println("Opening file: {}", m_path.c_str());
  file_mapping const m_file(m_path.c_str(), read_only);
  m_region = mapped_region(m_file, read_only);
  m_buffer = Buffer{2 * m_region.get_size()};// reserve double the size
  std::span<uint8_t> const dlt_span{static_cast<uint8_t *>(m_region.get_address()), m_region.get_size()};

  auto buffer_iter = m_buffer.begin();
  auto iterator = dlt_span.begin();
  auto const end = dlt_span.cend();

  while (iterator < end) {
    // The bit_cast from pointer to pointer are UB and the safe approach would be throw use
    // std::bit_cast to convert to DltStorageHeader, instead of DltStorageHeader*, but that
    // would copy everything and be more costly. Tests should be created to cover this unsafe code.

    // Storage header
    auto *const storage_header = std::bit_cast<DltStorageHeader *>(&*iterator);
    std::advance(iterator, sizeof(DltStorageHeader));
    m_patterns.emplace_back(storage_header->pattern, DLT_ID_SIZE);
    auto const _seconds = storage_header->seconds;
    m_seconds.emplace_back(_seconds);
    auto const _microseconds = storage_header->microseconds;
    m_microseconds.emplace_back(_microseconds);
    m_ecus.emplace_back(storage_header->ecu, DLT_ID_SIZE);

    if (dlt_check_storageheader(storage_header) != DLT_RETURN_TRUE) {
      throw std::runtime_error("Invalid DLT storage header");
    }

    // Standard header
    auto *const standard_header = std::bit_cast<DltStandardHeader *>(&*iterator);
    std::advance(iterator, sizeof(DltStandardHeader));
    auto const htyp = standard_header->htyp;
    m_htyps.emplace_back(htyp);
    auto const mcnt = standard_header->mcnt;
    m_mcnts.emplace_back(mcnt);
    auto const len = standard_header->len;
    m_lens.emplace_back(len);

    /* load standard header extra parameters if used */
    uint32_t seid = 0;
    uint32_t tmsp = 0;
    if (DLT_STANDARD_HEADER_EXTRA_SIZE(htyp)) {
      if (DLT_IS_HTYP_WEID(htyp)) {
        m_ecus.back() = std::string_view(std::bit_cast<const char *>(&*iterator), static_cast<size_t>(DLT_ID_SIZE));
        std::advance(iterator, DLT_ID_SIZE);
      }

      if (DLT_IS_HTYP_WSID(htyp)) {
        seid = DLT_BETOH_32(*std::bit_cast<uint32_t *>(&*iterator));
        std::advance(iterator, DLT_SIZE_WSID);
      }

      if (DLT_IS_HTYP_WTMS(htyp)) {
        std::memcpy(&tmsp, &*iterator, sizeof(tmsp));
        tmsp = DLT_BETOH_32(tmsp);
        std::advance(iterator, DLT_SIZE_WSID);
      }
    }
    m_seids.emplace_back(seid);
    m_tmsps.emplace_back(tmsp);

    /* set extended header ptr */
    uint8_t msin = 0;
    uint8_t noar = 0;
    std::string_view apid;
    std::string_view ctid;
    if (DLT_IS_HTYP_UEH(htyp)) {
      auto *const extended_header = std::bit_cast<DltExtendedHeader *>(&*iterator);
      std::advance(iterator, sizeof(DltExtendedHeader));
      msin = extended_header->msin;
      noar = extended_header->noar;
      apid = std::string_view{extended_header->apid, DLT_ID_SIZE};// NOLINT
      ctid = std::string_view{extended_header->ctid, DLT_ID_SIZE};// NOLINT
    }
    m_msins.emplace_back(msin);
    m_noars.emplace_back(noar);
    m_apids.emplace_back(apid);
    m_ctids.emplace_back(ctid);

    // Payload
    /* calculate complete size of headers */
    auto const header_size =
      static_cast<uint32_t>(sizeof(DltStorageHeader) + sizeof(DltStandardHeader) + DLT_STANDARD_HEADER_EXTRA_SIZE(htyp)
                            + (DLT_IS_HTYP_UEH(htyp) ? sizeof(DltExtendedHeader) : 0));

    /* calculate complete size of payload */
    // NOTE: cast to uint32_t needed because bitwise promotes unsigned types smaller than an int to an int
    int32_t const data_size = static_cast<int32_t>(DLT_BETOH_16(static_cast<uint32_t>(len)))
                              + static_cast<int32_t>(sizeof(DltStorageHeader)) - static_cast<int32_t>(header_size);

    auto buffer_iter_begin = buffer_iter;// save current buffer iterator to later construct the string_view
    std::string_view payload{std::bit_cast<char *>(&*iterator), static_cast<size_t>(data_size)};
    std::advance(iterator, data_size);
    std::string_view service_name;
    std::string_view return_type_name;
    if (_dlt_msg_is_nonverbose(htyp, msin)) {
      // non-verbose mode the payload buffer can be:
      // | service id name | return type | payload |

      // determine service id name
      auto const id_tmp = dlt_msg_read_value<uint32_t>(payload);
      auto const id_value = DLT_ENDIAN_GET_32(htyp, id_tmp);
      if (_dlt_msg_is_control(htyp, msin) && id_value < DLT_SERVICE_ID_LAST_ENTRY) {
        // Possible out of bounds if id > service_id_name.size()
        // The check is ignored in favor of  performance
        service_name = service_id_name[id_value];// NOLINT
      } else {
        m_buffer.store(id_value);
      }

      // determine return type name
      if (_dlt_msg_is_control_response(htyp, msin)) {
        auto retval = dlt_msg_read_value<uint8_t>(payload);
        // Possible out of bounds if id > service_id_name.size()
        // The check is ignored in favor of  performance
        return_type_name = return_type[retval];// NOLINT
      }
      buffer_iter = m_buffer.store(service_name, Hex(payload));

    } else {
      /* At this point, it is ensured that a extended header is available */

      // verbose mode the payload buffer can be:
      // | type info | payload | [ type_info | payload | ...]
      for (size_t _ = 0; _ < noar; ++_) {
        if (_ > 0) { m_buffer.store(' '); }// add a space in between arguments

        auto type_info_tmp = dlt_msg_read_value<uint32_t>(payload);
        uint32_t const type_info = DLT_ENDIAN_GET_32(htyp, type_info_tmp);

        if (((type_info & DLT_TYPE_INFO_STRG) != 0U)
            && (((type_info & DLT_TYPE_INFO_SCOD) == DLT_SCOD_ASCII)
                || ((type_info & DLT_TYPE_INFO_SCOD) == DLT_SCOD_UTF8))) {

          dlt_msg_read_value<uint16_t>(payload);
          buffer_iter = m_buffer.store(payload);

        } else if ((type_info & DLT_TYPE_INFO_BOOL) != 0U) {

          if ((type_info & DLT_TYPE_INFO_VARI) != 0U) { throw std::runtime_error("Not implemented yet"); }

          auto value = dlt_msg_read_value<bool>(payload);
          buffer_iter = m_buffer.store(value ? "true" : "false");

        } else if (((type_info & DLT_TYPE_INFO_SINT) != 0U) || ((type_info & DLT_TYPE_INFO_UINT) != 0U)) {

          if ((type_info & DLT_TYPE_INFO_VARI) != 0U) { throw std::runtime_error("Not implemented yet"); }

          if ((type_info & DLT_TYPE_INFO_FIXP) != 0U) { throw std::runtime_error("Not implemented yet"); }

          switch (type_info & DLT_TYPE_INFO_TYLE) {
          case DLT_TYLE_8BIT: {
            auto value = dlt_msg_read_value<uint8_t>(payload);
            buffer_iter = m_buffer.store(value);
            break;
          }
          case DLT_TYLE_16BIT: {
            if ((type_info & DLT_TYPE_INFO_SINT) != 0U) {
              auto value_tmp = static_cast<uint32_t>(dlt_msg_read_value<int16_t>(payload));
              auto value = static_cast<int16_t>(DLT_ENDIAN_GET_16(htyp, value_tmp));
              buffer_iter = m_buffer.store(value);
            } else {
              auto value_tmp = static_cast<uint32_t>(dlt_msg_read_value<uint16_t>(payload));
              buffer_iter = m_buffer.store(DLT_ENDIAN_GET_16(htyp, value_tmp));
            }
            break;
          }
          case DLT_TYLE_32BIT: {
            auto value_tmp = dlt_msg_read_value<uint32_t>(payload);
            buffer_iter = m_buffer.store(DLT_ENDIAN_GET_32(htyp, value_tmp));
            break;
          }
          case DLT_TYLE_64BIT: {
            auto value_tmp = dlt_msg_read_value<uint64_t>(payload);
            buffer_iter = m_buffer.store(DLT_ENDIAN_GET_64(htyp, value_tmp));
            break;
          }
          case DLT_TYLE_128BIT: {
            throw std::runtime_error("Not implemented yet");
          }
          default: {
            throw std::runtime_error("Unknown type info in DLT message");
          }
          }

        } else if ((type_info & DLT_TYPE_INFO_FLOA) != 0U) {
          if ((type_info & DLT_TYPE_INFO_VARI) != 0U) { throw std::runtime_error("Not implemented yet"); }

          switch (type_info & DLT_TYPE_INFO_TYLE) {
          case DLT_TYLE_8BIT: {
            auto value = dlt_msg_read_value<uint8_t>(payload);
            buffer_iter = m_buffer.store(value);
            break;
          }
          case DLT_TYLE_16BIT: {
            // auto value_tmp = dlt_msg_read_value<float>(payload);
            // payload = DLT_ENDIAN_GET_16(htyp, value_tmp);
            // payload = m_buffer.store(DLT_ENDIAN_GET_16(htyp, value_tmp));
            break;
          }
          case DLT_TYLE_32BIT: {
            auto value = dlt_msg_read_value<float32_t>(payload);
            auto value_uint32 = std::bit_cast<uint32_t>(value);
            auto value_uint32_swap = DLT_ENDIAN_GET_32(htyp, value_uint32);
            auto value_corrected = std::bit_cast<float32_t>(value_uint32_swap);
            buffer_iter = m_buffer.store(value_corrected);
            break;
          }
          case DLT_TYLE_64BIT: {
            auto value = dlt_msg_read_value<float64_t>(payload);
            auto value_uint64 = std::bit_cast<uint64_t>(value);
            auto value_uint64_swap = DLT_ENDIAN_GET_64(htyp, value_uint64);
            auto value_corrected = std::bit_cast<float64_t>(value_uint64_swap);
            buffer_iter = m_buffer.store(value_corrected);
            break;
          }
          case DLT_TYLE_128BIT: {
            throw std::runtime_error("Not implemented yet");
          }
          default: {
            throw std::runtime_error("Unknown type info in DLT message");
          }
          }
        } else if ((type_info & DLT_TYPE_INFO_RAWD) != 0U) {
          dlt_msg_read_value<uint16_t>(payload);
          buffer_iter = m_buffer.store(Hex(payload));
        }
      }
    }
    m_service_id_names.emplace_back(service_name);
    m_return_types.emplace_back(return_type_name);
    m_payloads.emplace_back(buffer_iter_begin, buffer_iter);

    m_size++;
  }

  assert(m_patterns.size() == m_size);
  assert(m_seconds.size() == m_size);
  assert(m_microseconds.size() == m_size);
  assert(m_ecus.size() == m_size);
  assert(m_htyps.size() == m_size);
  assert(m_mcnts.size() == m_size);
  assert(m_lens.size() == m_size);
  assert(m_seids.size() == m_size);
  assert(m_tmsps.size() == m_size);
  assert(m_msins.size() == m_size);
  assert(m_noars.size() == m_size);
  assert(m_apids.size() == m_size);
  assert(m_ctids.size() == m_size);
  assert(m_service_id_names.size() == m_size);
  assert(m_return_types.size() == m_size);
  assert(m_payloads.size() == m_size);
  assert(m_buffer.size() < m_buffer.capacity());
}

std::span<std::string_view const> DLT::patterns() const {
  return std::span<std::string_view const>{m_patterns.data(), m_size};
}

std::span<uint32_t const> DLT::seconds() const { return std::span<uint32_t const>{m_seconds.data(), m_size}; }

std::span<int32_t const> DLT::microseconds() const { return std::span<int32_t const>{m_microseconds.data(), m_size}; }

std::span<std::string_view const> DLT::ecus() const { return std::span<std::string_view const>{m_ecus.data(), m_size}; }

std::span<uint8_t const> DLT::header_types() const { return std::span<uint8_t const>{m_htyps.data(), m_size}; }

std::span<uint8_t const> DLT::message_counters() const { return std::span<uint8_t const>{m_mcnts.data(), m_size}; }

std::span<uint16_t const> DLT::lengths() const { return std::span<uint16_t const>{m_lens.data(), m_size}; }

std::span<uint32_t const> DLT::session_ids() const { return std::span<uint32_t const>{m_seids.data(), m_size}; }

std::span<uint32_t const> DLT::timestamps() const { return std::span<uint32_t const>{m_tmsps.data(), m_size}; }

std::span<uint8_t const> DLT::message_infos() const { return std::span<uint8_t const>{m_msins.data(), m_size}; }

std::span<uint8_t const> DLT::number_of_arguments() const { return std::span<uint8_t const>{m_noars.data(), m_size}; }

std::span<std::string_view const> DLT::app_ids() const {
  return std::span<std::string_view const>{m_apids.data(), m_size};
}

std::span<std::string_view const> DLT::ctx_ids() const {
  return std::span<std::string_view const>{m_ctids.data(), m_size};
}

std::span<std::string_view const> DLT::service_id_names() const {
  return std::span<std::string_view const>{m_service_id_names.data(), m_size};
}

std::span<std::string_view const> DLT::return_types() const {
  return std::span<std::string_view const>{m_return_types.data(), m_size};
}

std::span<std::string_view const> DLT::payloads() const {
  return std::span<std::string_view const>{m_payloads.data(), m_size};
}

size_t DLT::size() const { return m_size; }
