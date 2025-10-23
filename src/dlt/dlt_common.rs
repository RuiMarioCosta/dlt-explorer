// This file is meant to recreate the varriables, functions, macros, etc in dlt-daemon's
// dlt_common.h file so that it can be easier to update

use super::dlt_protocol::*;

/*
 * Macros to swap the byte order.
 */
// #   define DLT_SWAP_64(value) ((((uint64_t)DLT_SWAP_32((value) & 0xffffffffull)) << 32) | (DLT_SWAP_32((value) >> 32)))
//
// #   define DLT_SWAP_16(value) ((((value) >> 8) & 0xff) | (((value) << 8) & 0xff00))
// #   define DLT_SWAP_32(value) ((((value) >> 24) & 0xff) | (((value) << 8) & 0xff0000) | (((value) >> 8) & 0xff00) | \
// (((value) << 24) & 0xff000000))

fn dlt_swap_32(x: u32) -> u32 {
    ((x >> 24) & 0xff) | ((x << 8) & 0xff0000) | ((x >> 8) & 0xff00) | ((x << 24) & 0xff000000)
}

fn dlt_betoh_32(x: u32) -> u32 {
    dlt_swap_32(x)
}

fn dlt_letoh_32(x: u32) -> u32 {
    x
}

// #   define DLT_ENDIAN_GET_16(htyp, x) ((((htyp) & DLT_HTYP_MSBF) > 0) ? DLT_BETOH_16(x) : DLT_LETOH_16(x))
// #   define DLT_ENDIAN_GET_32(htyp, x) ((((htyp) & DLT_HTYP_MSBF) > 0) ? DLT_BETOH_32(x) : DLT_LETOH_32(x))
pub fn dlt_endian_get_32(htyp: u32, x: u32) -> u32 {
    if (htyp & DLT_HTYP_MSBF as u32) > 0 {
        dlt_betoh_32(x)
    } else {
        dlt_letoh_32(x)
    }
}

// #   define DLT_ENDIAN_GET_64(htyp, x) ((((htyp) & DLT_HTYP_MSBF) > 0) ? DLT_BETOH_64(x) : DLT_LETOH_64(x))

/**
 * The size of a DLT ID
 */
pub const DLT_ID_SIZE: usize = 4;

pub const DLT_SIZE_WEID: usize = DLT_ID_SIZE;
pub const DLT_SIZE_WSID: usize = 4; // size_of u32 in bytes
pub const DLT_SIZE_WTMS: usize = 4; // size of u32 in bytes

/**
 * Get the size of extra header parameters, depends on htyp.
 */
pub fn dlt_standard_header_extra_size(htyp: u8) -> usize {
    let mut size = 0;
    if dlt_is_htyp_weid(htyp) {
        size += DLT_SIZE_WEID;
    }
    if dlt_is_htyp_wsid(htyp) {
        size += DLT_SIZE_WSID;
    }
    if dlt_is_htyp_wtms(htyp) {
        size += DLT_SIZE_WTMS;
    }
    size
}

pub fn dlt_msg_is_control(htyp: u8, msin: u8) -> bool {
    dlt_is_htyp_ueh(htyp) && (dlt_get_msin_mstp(msin) == DLT_TYPE_CONTROL)
}

// #define DLT_MSG_IS_CONTROL_REQUEST(MSG)                                     \
//   ((DLT_IS_HTYP_UEH((MSG)->standardheader->htyp))                           \
//     && (DLT_GET_MSIN_MSTP((MSG)->extendedheader->msin) == DLT_TYPE_CONTROL) \
//     && (DLT_GET_MSIN_MTIN((MSG)->extendedheader->msin) == DLT_CONTROL_REQUEST))

pub fn dlt_msg_is_control_response(htyp: u8, msin: u8) -> bool {
    dlt_is_htyp_ueh(htyp)
        && (dlt_get_msin_mstp(msin) == DLT_TYPE_CONTROL)
        && (dlt_get_msin_mtin(msin) == DLT_CONTROL_RESPONSE)
}

// pub fn _dlt_msg_is_control_time(htyp: u8, msin: u8) -> bool {
//     dlt_is_htyp_ueh(htyp)
//         && (dlt_get_msin_mstp(msin) == DLT_TYPE_CONTROL)
//         && (dlt_get_msin_mtin(msin) == DLT_CONTROL_TIME)
// }

// #define DLT_MSG_IS_NW_TRACE(MSG)                  \
//   ((DLT_IS_HTYP_UEH((MSG)->standardheader->htyp)) \
//     && (DLT_GET_MSIN_MSTP((MSG)->extendedheader->msin) == DLT_TYPE_NW_TRACE))
//
// #define DLT_MSG_IS_TRACE_MOST(MSG)                                           \
//   ((DLT_IS_HTYP_UEH((MSG)->standardheader->htyp))                            \
//     && (DLT_GET_MSIN_MSTP((MSG)->extendedheader->msin) == DLT_TYPE_NW_TRACE) \
//     && (DLT_GET_MSIN_MTIN((MSG)->extendedheader->msin) == DLT_NW_TRACE_MOST))

pub fn dlt_msg_is_nonverbose(htyp: u8, msin: u8) -> bool {
    !dlt_is_htyp_ueh(htyp) || (dlt_is_htyp_ueh(htyp) && !dlt_is_msin_verb(msin))
}

pub const MESSAGE_TYPE: [&str; 8] = ["log", "app_trace", "nw_trace", "control", "", "", "", ""];
pub const LOG_INFO: [&str; 16] = [
    "", "fatal", "error", "warn", "info", "debug", "verbose", "", "", "", "", "", "", "", "", "",
];
// [[maybe_unused]] std::array<std::string_view, 16> const trace_type =
//   {"", "variable", "func_in", "func_out", "state", "vfb", "", "", "", "", "", "", "", "", "", ""};
// [[maybe_unused]] std::array<std::string_view, 16> const nw_trace_type =
//   {"", "ipc", "can", "flexray", "most", "vfb", "", "", "", "", "", "", "", "", "", ""};
pub const _CONTROL_TYPE: [&str; 16] = [
    "", "request", "response", "time", "", "", "", "", "", "", "", "", "", "", "", "",
];
pub const SERVICE_ID_NAME: [&str; 21] = [
    "",
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
    "message_buffer_overflow",
];
pub const RETURN_TYPE: [&str; 9] = [
    "ok",
    "not_supported",
    "error",
    "perm_denied",
    "warning",
    "",
    "",
    "",
    "no_matching_context_id",
];
