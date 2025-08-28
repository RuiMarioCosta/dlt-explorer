#pragma once

#include "adapter_export.h"

#include <filesystem>

#include "dlt_common.h"
#include "dlt_protocol.h"

[[maybe_unused]] extern const char *message_type[8];
[[maybe_unused]] extern const char *log_info[16];
[[maybe_unused]] extern const char *trace_type[16];
[[maybe_unused]] extern const char *nw_trace_type[16];
[[maybe_unused]] extern const char *control_type[16];

[[maybe_unused]] static const char *service_id_name[21] = {"",
  "set_log_level",
  "set_trace_status",
  "get_log_info",
  "get_default_log_level",
  "store_config",
  "reset_to_factory_default",
  "set_com_interface_status",
  "set_com_interface_max_bandwidth",
  "set_verbose_mode",
  "set_message_filtering",
  "set_timing_packets",
  "get_local_time",
  "use_ecu_id",
  "use_session_id",
  "use_timestamp",
  "use_extended_header",
  "set_default_log_level",
  "set_default_trace_status",
  "get_software_version",
  "message_buffer_overflow"};

[[maybe_unused]] static const char *return_type[9] =
  {"ok", "not_supported", "error", "perm_denied", "warning", "", "", "", "no_matching_context_id"};


/**
 * Parse DLT file using dlt-daemon functions
 */
void ADAPTER_EXPORT parse_dlt_daemon(std::filesystem::path const &path);
