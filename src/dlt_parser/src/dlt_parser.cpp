#include "dlt_parser.h"

#include "adapter.h"
#include "buffer.h"
#include "dlt_common.h"

#include <boost/interprocess/file_mapping.hpp>
#include <boost/interprocess/mapped_region.hpp>

#include <bit>
#include <chrono>
#include <filesystem>
#include <iostream>
#include <span>
#include <sstream>
#include <syslog.h>
#include <variant>
#include <vector>

using namespace boost::interprocess;

template<typename T> void print(std::span<T> data) {
  for (const auto &c : data) { std::cout << c; }
  std::cout << '\n';
}

struct RawData {
  std::string_view data;
};

struct Print {
  Print(std::ostream &os, bool non_verbose) : m_os{ os }, m_nonverbose{ non_verbose } {}

  void operator()(std::string_view payload) const {
    m_os << "string_view: [";
    if (m_nonverbose) {
      m_os << std::setfill('0') << std::hex;
      for (auto i : payload) { m_os << std::setw(2) << static_cast<int>(i) << ' '; }
      m_os << std::setfill(' ') << std::dec;
    } else {
      m_os << payload;
    }
    m_os << "]";
  }

  void operator()(RawData payload) const {
    m_os << " RawData: [" << std::setfill('0') << std::hex;
    for (auto i : payload.data) { m_os << std::setw(2) << static_cast<int>(i) << ' '; }
    m_os << std::setfill(' ') << std::dec << "]";
  }

  void operator()(auto payload) const { m_os << typeid(payload).name() << " auto: [" << payload << "]"; }

  std::ostream &m_os;
  bool m_nonverbose;
};

class DLT {
  using payload_t = std::variant<std::string_view,
    RawData,
    bool,
    int8_t,
    uint8_t,
    int16_t,
    uint16_t,
    int32_t,
    uint32_t,
    int64_t,
    uint64_t,
    float32_t,
    float64_t>;

  static_assert(sizeof(payload_t) == 24);
  static_assert(alignof(payload_t) == 8);

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
  // The size of a variant is the size of the largest type it can hold, plus
  // the alignment requirement of the type with the biggest aligmnent because it
  // needs something to know what is the active alternative. In this case it is
  // sizeof(string_view) + alignment = 16 + 8 = 24.
  // std::vector<payload_t> m_payloads;
  std::vector<std::string_view> m_payloads;

  Buffer m_buffer;

  // invariant: sizes of all vectors must be the same
  size_t m_size;
  mapped_region m_region;

public:
  DLT() = delete;
  DLT(mapped_region region) : m_region(std::move(region)), m_size(0), m_buffer(1024) {
    assert(m_buffer.capacity() == 1024);

    uint8_t const *const begin = static_cast<uint8_t *>(m_region.get_address());
    uint8_t const *const end = begin + m_region.get_size();
    auto buffer = m_region.get_address();

    while (static_cast<uint8_t *>(buffer) < end) {
      // Storage header
      auto const storageHeader = static_cast<DltStorageHeader *>(buffer);
      buffer = static_cast<uint8_t *>(buffer) + sizeof(DltStorageHeader);
      m_patterns.emplace_back(std::string_view{ storageHeader->pattern, DLT_ID_SIZE });
      auto const s = storageHeader->seconds;
      m_seconds.emplace_back(s);
      auto const ms = storageHeader->microseconds;
      m_microseconds.emplace_back(ms);
      m_ecus.emplace_back(std::string_view{ storageHeader->ecu, DLT_ID_SIZE });

      if (dlt_check_storageheader(storageHeader) != DLT_RETURN_TRUE) {
        throw std::runtime_error("Invalid DLT storage header");
      }

      // Standard header
      auto const standardHeader = static_cast<DltStandardHeader *>(buffer);
      buffer = static_cast<uint8_t *>(buffer) + sizeof(DltStandardHeader);
      auto const htyp = standardHeader->htyp;
      m_htyps.emplace_back(htyp);
      auto const mcnt = standardHeader->mcnt;
      m_mcnts.emplace_back(mcnt);
      auto const len = standardHeader->len;
      m_lens.emplace_back(len);

      /* load standard header extra parameters if used */
      if (DLT_STANDARD_HEADER_EXTRA_SIZE(htyp)) {
        if (DLT_IS_HTYP_WEID(htyp)) {
          m_ecus.back() = std::string_view(static_cast<char *>(buffer), static_cast<size_t>(DLT_ID_SIZE));
          buffer = static_cast<uint8_t *>(buffer) + DLT_ID_SIZE;
        }

        if (DLT_IS_HTYP_WSID(htyp)) {
          m_seids.emplace_back(DLT_BETOH_32(*static_cast<uint32_t *>(buffer)));
          buffer = static_cast<uint8_t *>(buffer) + DLT_SIZE_WSID;
        } else {
          m_seids.emplace_back(0);
        }

        if (DLT_IS_HTYP_WTMS(htyp)) {
          m_tmsps.emplace_back(DLT_BETOH_32(*static_cast<uint32_t *>(buffer)));
          buffer = static_cast<uint8_t *>(buffer) + DLT_SIZE_WTMS;
        } else {
          m_tmsps.emplace_back(0);
        }
      } else {
        m_seids.emplace_back(0);
        m_tmsps.emplace_back(0);
      }

      /* set extended header ptr */
      uint8_t msin = 0;
      uint8_t noar = 0;
      std::string_view apid = "";
      std::string_view ctid = "";
      if (DLT_IS_HTYP_UEH(htyp)) {
        auto const extendedHeader = static_cast<DltExtendedHeader *>(buffer);
        msin = extendedHeader->msin;
        noar = extendedHeader->noar;
        apid = std::string_view{ extendedHeader->apid, DLT_ID_SIZE };
        ctid = std::string_view{ extendedHeader->ctid, DLT_ID_SIZE };
        buffer = static_cast<uint8_t *>(buffer) + sizeof(DltExtendedHeader);
      }
      m_msins.emplace_back(msin);
      m_noars.emplace_back(noar);
      m_apids.emplace_back(apid);
      m_ctids.emplace_back(ctid);

      // Payload
      /* calculate complete size of headers */
      auto const headerSize =
        (uint32_t)(sizeof(DltStorageHeader) + sizeof(DltStandardHeader) + DLT_STANDARD_HEADER_EXTRA_SIZE(htyp)
                   + (DLT_IS_HTYP_UEH(htyp) ? sizeof(DltExtendedHeader) : 0));

      /* calculate complete size of payload */
      int32_t const dataSize = DLT_BETOH_16(len) + (int32_t)sizeof(DltStorageHeader) - static_cast<int32_t>(headerSize);

      std::string_view payload{ static_cast<char *>(buffer), static_cast<size_t>(dataSize) };
      std::string_view service_name_id = "";
      std::string_view return_type_name = "";
      // non-verbose mode the payload buffer can be:
      // | service id name | return type | payload |
      if (DLT_IS_HTYP_UEH(htyp) && _dlt_msg_is_nonverbose(htyp, msin)) {
        // determine service id name
        auto id_tmp = _dlt_msg_read_value<uint32_t>(payload);
        auto id = DLT_ENDIAN_GET_32(htyp, id_tmp);
        if (_dlt_msg_is_control(htyp, msin) && id < DLT_SERVICE_ID_LAST_ENTRY) {
          // Possible out of bounds if id > service_id_name.size()
          // The check is ignored in favor of  performance
          service_name_id = service_id_name[id];
        }

        // determine return type name
        if (_dlt_msg_is_control_response(htyp, msin)) {
          auto retval = _dlt_msg_read_value<uint8_t>(payload);
          // Possible out of bounds if id > service_id_name.size()
          // The check is ignored in favor of  performance
          return_type_name = return_type[retval];
        }
        // payload = payload;
        payload = m_buffer.store(service_id_name + payload);
      } else {
        /* At this point, it is ensured that a extended header is available */

        // verbose mode the payload buffer can be:
        // | type info | payload |
        for (size_t n = 0; n < noar; ++n) {
          auto type_info_tmp = _dlt_msg_read_value<uint32_t>(payload);
          uint32_t type_info = DLT_ENDIAN_GET_32(htyp, type_info_tmp);

          if ((type_info & DLT_TYPE_INFO_STRG)
              && (((type_info & DLT_TYPE_INFO_SCOD) == DLT_SCOD_ASCII)
                  || ((type_info & DLT_TYPE_INFO_SCOD) == DLT_SCOD_UTF8))) {

            auto value = _dlt_msg_read_value<uint16_t>(payload);
            payload = payload;

          } else if (type_info & DLT_TYPE_INFO_BOOL) {

            if (type_info & DLT_TYPE_INFO_VARI) { throw std::runtime_error("Not implemented yet"); }

            auto value = _dlt_msg_read_value<bool>(payload);
            payload = value ? "true" : "false";

          } else if ((type_info & DLT_TYPE_INFO_SINT) || (type_info & DLT_TYPE_INFO_UINT)) {

            if (type_info & DLT_TYPE_INFO_VARI) { throw std::runtime_error("Not implemented yet"); }

            if (type_info & DLT_TYPE_INFO_FIXP) { throw std::runtime_error("Not implemented yet"); }

            switch (type_info & DLT_TYPE_INFO_TYLE) {
            case DLT_TYLE_8BIT: {
              auto value = _dlt_msg_read_value<uint8_t>(payload);
              // payload = static_cast<int64_t>(value);
              payload = m_buffer.store(value);
              break;
            }
            case DLT_TYLE_16BIT: {
              auto value_tmp = _dlt_msg_read_value<uint16_t>(payload);
              // payload = DLT_ENDIAN_GET_16(htyp, value_tmp);
              payload = m_buffer.store(DLT_ENDIAN_GET_16(htyp, value_tmp));
              break;
            }
            case DLT_TYLE_32BIT: {
              auto value_tmp = _dlt_msg_read_value<uint32_t>(payload);
              // payload = DLT_ENDIAN_GET_32(htyp, value_tmp);
              payload = m_buffer.store(DLT_ENDIAN_GET_32(htyp, value_tmp));
              break;
            }
            case DLT_TYLE_64BIT: {
              auto value_tmp = _dlt_msg_read_value<uint64_t>(payload);
              // payload = DLT_ENDIAN_GET_64(htyp, value_tmp);
              payload = m_buffer.store(DLT_ENDIAN_GET_64(htyp, value_tmp));
              break;
            }
            case DLT_TYLE_128BIT: {
              throw std::runtime_error("Not implemented yet");
              break;
            }
            default: {
              throw std::runtime_error("Unknown type info in DLT message");
            }
            }

          } else if (type_info & DLT_TYPE_INFO_FLOA) {
            if (type_info & DLT_TYPE_INFO_VARI) { throw std::runtime_error("Not implemented yet"); }

            switch (type_info & DLT_TYPE_INFO_TYLE) {
            case DLT_TYLE_8BIT: {
              auto value = _dlt_msg_read_value<uint8_t>(payload);
              // payload = static_cast<int64_t>(value);
              payload = m_buffer.store(value);
              break;
            }
            case DLT_TYLE_16BIT: {
              auto value_tmp = _dlt_msg_read_value<float>(payload);
              // payload = DLT_ENDIAN_GET_16(htyp, value_tmp);
              // payload = m_buffer.store(DLT_ENDIAN_GET_16(htyp, value_tmp));
              break;
            }
            case DLT_TYLE_32BIT: {
              auto value = _dlt_msg_read_value<float32_t>(payload);
              auto value_int32 = std::bit_cast<int32_t>(value);
              auto value_int32_swap = DLT_ENDIAN_GET_32(htyp, value_int32);
              auto value_corrected = std::bit_cast<float32_t>(value_int32_swap);
              // payload = value_corrected;
              payload = m_buffer.store(value_corrected);
              break;
            }
            case DLT_TYLE_64BIT: {
              auto value = _dlt_msg_read_value<float64_t>(payload);
              auto value_int64 = std::bit_cast<int64_t>(value);
              auto value_int64_swap = DLT_ENDIAN_GET_64(htyp, value_int64);
              auto value_corrected = std::bit_cast<float64_t>(value_int64_swap);
              // payload = value_corrected;
              payload = m_buffer.store(value_corrected);
              break;
            }
            case DLT_TYLE_128BIT: {
              throw std::runtime_error("Not implemented yet");
              break;
            }
            default: {
              throw std::runtime_error("Unknown type info in DLT message");
            }
            }
          } else if (type_info & DLT_TYPE_INFO_RAWD) {
            auto value = _dlt_msg_read_value<uint16_t>(payload);
            // payload = RawData{payload};
            payload = m_buffer.store(value);
          }
        }
      }
      m_service_id_names.emplace_back(service_name_id);
      m_return_types.emplace_back(return_type_name);
      m_payloads.emplace_back(payload);

      buffer = static_cast<uint8_t *>(buffer) + dataSize;

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
  }

  void print() {
    std::ostringstream os;
    for (size_t i = 0; i < m_size; ++i) {
      auto mt = message_type[DLT_GET_MSIN_MSTP(static_cast<int>(m_msins.at(i)))];
      auto id = DLT_GET_MSIN_MTIN(static_cast<int>(m_msins.at(i)));
      auto li = log_info[id];
      auto tt = trace_type[id];
      auto ntt = nw_trace_type[id];
      auto ct = control_type[id];

      // os << "Pattern: " << dlt.m_patterns.at(i) << ", ";
      os << "Seconds: " << m_seconds.at(i) << ", ";
      os << "Microseconds: " << m_microseconds.at(i) << ", ";
      os << "ECU: " << m_ecus.at(i) << ", ";
      os << "HTYP: " << static_cast<int>(m_htyps.at(i)) << ", ";
      os << "MCNT: " << static_cast<int>(m_mcnts.at(i)) << ", ";
      os << "Length: " << m_lens.at(i) << " " << DLT_BETOH_16(m_lens.at(i)) << ", ";
      os << "SEID: " << m_seids.at(i) << ", ";
      os << "TMSP: " << m_tmsps.at(i) << ", ";
      os << "MSIN: (" << mt << ", " << li << ", " << tt << ", " << ntt << ", " << ct << "), ";
      os << "NOAR: " << static_cast<int>(m_noars.at(i)) << ", ";
      os << "APID: " << m_apids.at(i) << ", ";
      os << "CTID: " << m_ctids.at(i) << ", ";
      // os << '\n';
      os << "Payload: ";
      os << m_service_id_names.at(i) << ", ";
      os << m_return_types.at(i) << ", ";
      os << m_payloads.at(i) << ", ";
      // std::visit(
      //     Print{os, _dlt_msg_is_nonverbose(m_htyps.at(i), m_msins.at(i))},
      //     m_payloads.at(i));
      os << '\n';
    }
    std::cout << os.str();
  }

  friend std::ostream &operator<<(std::ostream &os, DLT const &dlt) {
    os << std::boolalpha;
    for (size_t i = 0; i < dlt.m_size; ++i) {
      auto id = DLT_GET_MSIN_MSTP(static_cast<int>(dlt.m_msins.at(i)));
      auto mt = message_type[id];
      auto li = log_info[id];
      auto tt = trace_type[id];
      auto ntt = nw_trace_type[id];
      auto ct = control_type[id];

      auto time = std::chrono::seconds{ dlt.m_seconds.at(i) } + std::chrono::microseconds{ dlt.m_microseconds.at(i) };
      std::chrono::system_clock::time_point tp{ time };
      os << tp << "|";
      os << std::setw(10) << dlt.m_tmsps.at(i) << "|";
      os << std::setw(6) << static_cast<int>(dlt.m_mcnts.at(i)) << "|";
      os << std::setw(4) << dlt.m_ecus.at(i) << "|";
      os << std::setw(4) << dlt.m_apids.at(i) << "|";
      os << std::setw(4) << dlt.m_ctids.at(i) << "|";
      os << std::setw(10) << message_type[DLT_GET_MSIN_MSTP(static_cast<int>(dlt.m_msins.at(i)))] << "|";
      os << std::setw(10) << log_info[DLT_GET_MSIN_MTIN(static_cast<int>(dlt.m_msins.at(i)))] << "|";
      if (dlt._dlt_msg_is_nonverbose(dlt.m_htyps.at(i), dlt.m_msins.at(i))) {
        os << "N";
      } else {
        os << "V";
      }
      os << dlt.m_noars.at(i) << "|";
      os << dlt.m_payloads.at(i) << "|";
      // std::visit(Print{os, dlt._dlt_msg_is_nonverbose(dlt.m_htyps.at(i),
      //                                                 dlt.m_msins.at(i))},
      //            dlt.m_payloads.at(i));
      os << '\n';
    }
    return os;
  }

private:
  bool _dlt_msg_is_nonverbose(int htyp, int msin) const {
    return (!DLT_IS_HTYP_UEH(htyp) || (DLT_IS_HTYP_UEH(htyp) && !DLT_IS_MSIN_VERB(msin)));
  }

  bool _dlt_msg_is_control(int htyp, int msin) const {
    return DLT_IS_HTYP_UEH(htyp) && (DLT_GET_MSIN_MSTP(msin) == DLT_TYPE_CONTROL);
  }

  bool _dlt_msg_is_control_response(int htyp, int msin) const {
    return DLT_IS_HTYP_UEH(htyp) && (DLT_GET_MSIN_MSTP(msin) == DLT_TYPE_CONTROL)
           && (DLT_GET_MSIN_MTIN(msin) == DLT_CONTROL_RESPONSE);
  }

  template<typename T> T _dlt_msg_read_value(std::string_view &payload) {
    if (payload.size() < sizeof(T)) { throw std::runtime_error("Payload size is less than expected"); }
    auto dst = *reinterpret_cast<T *>(const_cast<char *>(payload.data()));
    payload.remove_prefix(sizeof(T));
    return dst;
  }
};

void parse_dlt_explorer(std::filesystem::path const &path) {
  file_mapping m_file(path.c_str(), read_only);

  mapped_region region(m_file, read_only);

  DLT dlt{ std::move(region) };
  std::cout << dlt << '\n';
  // dlt.print();
}
