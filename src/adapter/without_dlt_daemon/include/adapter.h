#pragma once

#include "adapter_export.h"

#include <array>
#include <filesystem>
#include <string_view>


[[maybe_unused]] std::array<std::string_view, 8> const message_type =
  {"log", "app_trace", "nw_trace", "control", "", "", "", ""};

[[maybe_unused]] std::array<std::string_view, 16> const log_info =
  {"", "fatal", "error", "warn", "info", "debug", "verbose", "", "", "", "", "", "", "", "", ""};

[[maybe_unused]] std::array<std::string_view, 16> const trace_type =
  {"", "variable", "func_in", "func_out", "state", "vfb", "", "", "", "", "", "", "", "", "", ""};

[[maybe_unused]] std::array<std::string_view, 16> const nw_trace_type =
  {"", "ipc", "can", "flexray", "most", "vfb", "", "", "", "", "", "", "", "", "", ""};

[[maybe_unused]] std::array<std::string_view, 16> const control_type =
  {"", "request", "response", "time", "", "", "", "", "", "", "", "", "", "", "", ""};

[[maybe_unused]] std::array<std::string_view, 21> const service_id_name = {"",
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

[[maybe_unused]] std::array<std::string_view, 21> const return_type =
  {"ok", "not_supported", "error", "perm_denied", "warning", "", "", "", "no_matching_context_id"};


/**
 * Parse DLT file as done in dlt-daemon
 */
void ADAPTER_EXPORT parse_dlt_daemon(std::filesystem::path const &path);
