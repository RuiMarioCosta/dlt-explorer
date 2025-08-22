#include "dlt_parser.h"

// #include "adapter.h"
#include "buffer.h"
// #include "dlt_common.h"
// #include "dlt_protocol.h"

#include <boost/interprocess/detail/os_file_functions.hpp>
#include <boost/interprocess/file_mapping.hpp>
#include <boost/interprocess/mapped_region.hpp>

// #include <bit>
#include <cassert>
// #include <chrono>
// #include <cstddef>
#include <cstdint>
#include <filesystem>
// #include <iomanip>
// #include <iostream>
#include <span>
#include <utility>
// #include <sstream>
// #include <string_view>
#include <syslog.h>
// #include <vector>


using namespace boost::interprocess;

namespace {

// [[nodiscard]] bool _dlt_msg_is_nonverbose(int htyp, int msin) {
//   return (!DLT_IS_HTYP_UEH(htyp) || (DLT_IS_HTYP_UEH(htyp) && !DLT_IS_MSIN_VERB(msin)));
// }

// bool _dlt_msg_is_control(int htyp, int msin) const {
//   return DLT_IS_HTYP_UEH(htyp) && (DLT_GET_MSIN_MSTP(msin) == DLT_TYPE_CONTROL);
// }
//
// bool _dlt_msg_is_control_response(int htyp, int msin) const {
//   return DLT_IS_HTYP_UEH(htyp) && (DLT_GET_MSIN_MSTP(msin) == DLT_TYPE_CONTROL)
//          && (DLT_GET_MSIN_MTIN(msin) == DLT_CONTROL_RESPONSE);
// }
//
// template<typename T> T _dlt_msg_read_value(std::string_view &payload) {
//   if (payload.size() < sizeof(T)) { throw std::runtime_error("Payload size is less than expected"); }
//   auto dst = *reinterpret_cast<T *>(const_cast<char *>(payload.data()));
//   payload.remove_prefix(sizeof(T));
//   return dst;
// }

}// namespace

DLT::DLT(std::filesystem::path path) : m_path{std::move(path)} {
  file_mapping const m_file(m_path.c_str(), read_only);
  m_region = mapped_region(m_file, read_only);
  m_buffer = Buffer{m_region.get_size()};
  std::span<uint8_t> const dlt_span{static_cast<uint8_t *>(m_region.get_address()), m_region.get_size()};

  auto iterator = dlt_span.begin();
  auto const end = dlt_span.cend();

  while (iterator < end) {
    // Storage header
    // auto const storageHeader = static_cast<DltStorageHeader *>(buffer);
    // buffer = static_cast<uint8_t *>(buffer) + sizeof(DltStorageHeader);
    // m_patterns.emplace_back(std::string_view{ storageHeader->pattern, DLT_ID_SIZE });
    // auto const s = storageHeader->seconds;
    // m_seconds.emplace_back(s);
    // auto const ms = storageHeader->microseconds;
    // m_microseconds.emplace_back(ms);
    // m_ecus.emplace_back(std::string_view{ storageHeader->ecu, DLT_ID_SIZE });
    //
    // if (dlt_check_storageheader(storageHeader) != DLT_RETURN_TRUE) {
    //   throw std::runtime_error("Invalid DLT storage header");
    // }

    //   // Standard header
    //   auto const standardHeader = static_cast<DltStandardHeader *>(buffer);
    //   buffer = static_cast<uint8_t *>(buffer) + sizeof(DltStandardHeader);
    //   auto const htyp = standardHeader->htyp;
    //   m_htyps.emplace_back(htyp);
    //   auto const mcnt = standardHeader->mcnt;
    //   m_mcnts.emplace_back(mcnt);
    //   auto const len = standardHeader->len;
    //   m_lens.emplace_back(len);
    //
    //   /* load standard header extra parameters if used */
    //   if (DLT_STANDARD_HEADER_EXTRA_SIZE(htyp)) {
    //     if (DLT_IS_HTYP_WEID(htyp)) {
    //       m_ecus.back() = std::string_view(static_cast<char *>(buffer), static_cast<size_t>(DLT_ID_SIZE));
    //       buffer = static_cast<uint8_t *>(buffer) + DLT_ID_SIZE;
    //     }
    //
    //     if (DLT_IS_HTYP_WSID(htyp)) {
    //       m_seids.emplace_back(DLT_BETOH_32(*static_cast<uint32_t *>(buffer)));
    //       buffer = static_cast<uint8_t *>(buffer) + DLT_SIZE_WSID;
    //     } else {
    //       m_seids.emplace_back(0);
    //     }
    //
    //     if (DLT_IS_HTYP_WTMS(htyp)) {
    //       m_tmsps.emplace_back(DLT_BETOH_32(*static_cast<uint32_t *>(buffer)));
    //       buffer = static_cast<uint8_t *>(buffer) + DLT_SIZE_WTMS;
    //     } else {
    //       m_tmsps.emplace_back(0);
    //     }
    //   } else {
    //     m_seids.emplace_back(0);
    //     m_tmsps.emplace_back(0);
    //   }
    //
    //   /* set extended header ptr */
    //   uint8_t msin = 0;
    //   uint8_t noar = 0;
    //   std::string_view apid = "";
    //   std::string_view ctid = "";
    //   if (DLT_IS_HTYP_UEH(htyp)) {
    //     auto const extendedHeader = static_cast<DltExtendedHeader *>(buffer);
    //     msin = extendedHeader->msin;
    //     noar = extendedHeader->noar;
    //     apid = std::string_view{ extendedHeader->apid, DLT_ID_SIZE };
    //     ctid = std::string_view{ extendedHeader->ctid, DLT_ID_SIZE };
    //     buffer = static_cast<uint8_t *>(buffer) + sizeof(DltExtendedHeader);
    //   }
    //   m_msins.emplace_back(msin);
    //   m_noars.emplace_back(noar);
    //   m_apids.emplace_back(apid);
    //   m_ctids.emplace_back(ctid);
    //
    //   // Payload
    //   /* calculate complete size of headers */
    //   auto const headerSize =
    //     (uint32_t)(sizeof(DltStorageHeader) + sizeof(DltStandardHeader) + DLT_STANDARD_HEADER_EXTRA_SIZE(htyp)
    //                + (DLT_IS_HTYP_UEH(htyp) ? sizeof(DltExtendedHeader) : 0));
    //
    //   /* calculate complete size of payload */
    //   int32_t const dataSize = DLT_BETOH_16(len) + (int32_t)sizeof(DltStorageHeader) -
    //   static_cast<int32_t>(headerSize);
    //
    //   std::string_view payload{ static_cast<char *>(buffer), static_cast<size_t>(dataSize) };
    //   std::string_view service_name_id = "";
    //   std::string_view return_type_name = "";
    //   // non-verbose mode the payload buffer can be:
    //   // | service id name | return type | payload |
    //   if (DLT_IS_HTYP_UEH(htyp) && _dlt_msg_is_nonverbose(htyp, msin)) {
    //     // determine service id name
    //     auto id_tmp = _dlt_msg_read_value<uint32_t>(payload);
    //     auto id = DLT_ENDIAN_GET_32(htyp, id_tmp);
    //     if (_dlt_msg_is_control(htyp, msin) && id < DLT_SERVICE_ID_LAST_ENTRY) {
    //       // Possible out of bounds if id > service_id_name.size()
    //       // The check is ignored in favor of  performance
    //       service_name_id = service_id_name[id];
    //     }
    //
    //     // determine return type name
    //     if (_dlt_msg_is_control_response(htyp, msin)) {
    //       auto retval = _dlt_msg_read_value<uint8_t>(payload);
    //       // Possible out of bounds if id > service_id_name.size()
    //       // The check is ignored in favor of  performance
    //       return_type_name = return_type[retval];
    //     }
    //     // payload = payload;
    //     payload = m_buffer.store(service_id_name + payload);
    //   } else {
    //     /* At this point, it is ensured that a extended header is available */
    //
    //     // verbose mode the payload buffer can be:
    //     // | type info | payload |
    //     for (size_t n = 0; n < noar; ++n) {
    //       auto type_info_tmp = _dlt_msg_read_value<uint32_t>(payload);
    //       uint32_t type_info = DLT_ENDIAN_GET_32(htyp, type_info_tmp);
    //
    //       if ((type_info & DLT_TYPE_INFO_STRG)
    //           && (((type_info & DLT_TYPE_INFO_SCOD) == DLT_SCOD_ASCII)
    //               || ((type_info & DLT_TYPE_INFO_SCOD) == DLT_SCOD_UTF8))) {
    //
    //         auto value = _dlt_msg_read_value<uint16_t>(payload);
    //         payload = payload;
    //
    //       } else if (type_info & DLT_TYPE_INFO_BOOL) {
    //
    //         if (type_info & DLT_TYPE_INFO_VARI) { throw std::runtime_error("Not implemented yet"); }
    //
    //         auto value = _dlt_msg_read_value<bool>(payload);
    //         payload = value ? "true" : "false";
    //
    //       } else if ((type_info & DLT_TYPE_INFO_SINT) || (type_info & DLT_TYPE_INFO_UINT)) {
    //
    //         if (type_info & DLT_TYPE_INFO_VARI) { throw std::runtime_error("Not implemented yet"); }
    //
    //         if (type_info & DLT_TYPE_INFO_FIXP) { throw std::runtime_error("Not implemented yet"); }
    //
    //         switch (type_info & DLT_TYPE_INFO_TYLE) {
    //         case DLT_TYLE_8BIT: {
    //           auto value = _dlt_msg_read_value<uint8_t>(payload);
    //           // payload = static_cast<int64_t>(value);
    //           payload = m_buffer.store(value);
    //           break;
    //         }
    //         case DLT_TYLE_16BIT: {
    //           auto value_tmp = _dlt_msg_read_value<uint16_t>(payload);
    //           // payload = DLT_ENDIAN_GET_16(htyp, value_tmp);
    //           payload = m_buffer.store(DLT_ENDIAN_GET_16(htyp, value_tmp));
    //           break;
    //         }
    //         case DLT_TYLE_32BIT: {
    //           auto value_tmp = _dlt_msg_read_value<uint32_t>(payload);
    //           // payload = DLT_ENDIAN_GET_32(htyp, value_tmp);
    //           payload = m_buffer.store(DLT_ENDIAN_GET_32(htyp, value_tmp));
    //           break;
    //         }
    //         case DLT_TYLE_64BIT: {
    //           auto value_tmp = _dlt_msg_read_value<uint64_t>(payload);
    //           // payload = DLT_ENDIAN_GET_64(htyp, value_tmp);
    //           payload = m_buffer.store(DLT_ENDIAN_GET_64(htyp, value_tmp));
    //           break;
    //         }
    //         case DLT_TYLE_128BIT: {
    //           throw std::runtime_error("Not implemented yet");
    //           break;
    //         }
    //         default: {
    //           throw std::runtime_error("Unknown type info in DLT message");
    //         }
    //         }
    //
    //       } else if (type_info & DLT_TYPE_INFO_FLOA) {
    //         if (type_info & DLT_TYPE_INFO_VARI) { throw std::runtime_error("Not implemented yet"); }
    //
    //         switch (type_info & DLT_TYPE_INFO_TYLE) {
    //         case DLT_TYLE_8BIT: {
    //           auto value = _dlt_msg_read_value<uint8_t>(payload);
    //           // payload = static_cast<int64_t>(value);
    //           payload = m_buffer.store(value);
    //           break;
    //         }
    //         case DLT_TYLE_16BIT: {
    //           auto value_tmp = _dlt_msg_read_value<float>(payload);
    //           // payload = DLT_ENDIAN_GET_16(htyp, value_tmp);
    //           // payload = m_buffer.store(DLT_ENDIAN_GET_16(htyp, value_tmp));
    //           break;
    //         }
    //         case DLT_TYLE_32BIT: {
    //           auto value = _dlt_msg_read_value<float32_t>(payload);
    //           auto value_int32 = std::bit_cast<int32_t>(value);
    //           auto value_int32_swap = DLT_ENDIAN_GET_32(htyp, value_int32);
    //           auto value_corrected = std::bit_cast<float32_t>(value_int32_swap);
    //           // payload = value_corrected;
    //           payload = m_buffer.store(value_corrected);
    //           break;
    //         }
    //         case DLT_TYLE_64BIT: {
    //           auto value = _dlt_msg_read_value<float64_t>(payload);
    //           auto value_int64 = std::bit_cast<int64_t>(value);
    //           auto value_int64_swap = DLT_ENDIAN_GET_64(htyp, value_int64);
    //           auto value_corrected = std::bit_cast<float64_t>(value_int64_swap);
    //           // payload = value_corrected;
    //           payload = m_buffer.store(value_corrected);
    //           break;
    //         }
    //         case DLT_TYLE_128BIT: {
    //           throw std::runtime_error("Not implemented yet");
    //           break;
    //         }
    //         default: {
    //           throw std::runtime_error("Unknown type info in DLT message");
    //         }
    //         }
    //       } else if (type_info & DLT_TYPE_INFO_RAWD) {
    //         auto value = _dlt_msg_read_value<uint16_t>(payload);
    //         // payload = RawData{payload};
    //         payload = m_buffer.store(value);
    //       }
    //     }
    //   }
    //   m_service_id_names.emplace_back(service_name_id);
    //   m_return_types.emplace_back(return_type_name);
    //   m_payloads.emplace_back(payload);
    //
    //   buffer = static_cast<uint8_t *>(buffer) + dataSize;
    //
    m_size++;
    break;
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
}

// std::ostream &operator<<(std::ostream &ostream, DLT const &dlt) {
//   constexpr int TMSPS_WIDTH = 10;
//   constexpr int MCNTS_WIDTH = 10;
//   constexpr int MSTP_WIDTH = 10;
//   constexpr int LOG_WIDTH = 10;
//   ostream << std::boolalpha;
//   for (size_t i = 0; i < dlt.m_size; ++i) {
//
//     auto time = std::chrono::seconds{dlt.m_seconds.at(i)} + std::chrono::microseconds{dlt.m_microseconds.at(i)};
//     std::chrono::system_clock::time_point const time_point{time};
//     ostream << time_point << "|";
//     ostream << std::setw(TMSPS_WIDTH) << dlt.m_tmsps.at(i) << "|";
//     ostream << std::setw(MCNTS_WIDTH) << static_cast<int>(dlt.m_mcnts.at(i)) << "|";
//     ostream << std::setw(4) << dlt.m_ecus.at(i) << "|";
//     ostream << std::setw(4) << dlt.m_apids.at(i) << "|";
//     ostream << std::setw(4) << dlt.m_ctids.at(i) << "|";
//     // NOLINTBEGIN (cppcoreguidelines-pro-bounds-constant-array-index)
//     ostream << std::setw(MSTP_WIDTH) << message_type[DLT_GET_MSIN_MSTP(static_cast<int>(dlt.m_msins.at(i)))] << "|";
//     ostream << std::setw(LOG_WIDTH) << log_info[DLT_GET_MSIN_MTIN(static_cast<int>(dlt.m_msins.at(i)))] << "|";
//     // NOLINTEND (cppcoreguidelines-pro-bounds-constant-array-index)
//     if (dlt._dlt_msg_is_nonverbose(dlt.m_htyps.at(i), dlt.m_msins.at(i))) {
//       ostream << "N";
//     } else {
//       ostream << "V";
//     }
//     ostream << dlt.m_noars.at(i) << "|";
//     ostream << dlt.m_payloads.at(i) << "|";
//     ostream << '\n';
//   }
//   return ostream;
// }
