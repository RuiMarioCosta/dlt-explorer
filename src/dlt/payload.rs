// ---------------------------------------------------------------------------
// Payload type-info constants (from AUTOSAR PRS)
// ---------------------------------------------------------------------------

pub const DLT_TYPE_INFO_TYLE: u32 = 0x0000000f;
pub const DLT_TYPE_INFO_BOOL: u32 = 0x00000010;
pub const DLT_TYPE_INFO_SINT: u32 = 0x00000020;
pub const DLT_TYPE_INFO_UINT: u32 = 0x00000040;
pub const DLT_TYPE_INFO_FLOA: u32 = 0x00000080;
pub const DLT_TYPE_INFO_STRG: u32 = 0x00000200;
pub const DLT_TYPE_INFO_RAWD: u32 = 0x00000400;
pub const DLT_TYPE_INFO_VARI: u32 = 0x00000800;
pub const DLT_TYPE_INFO_FIXP: u32 = 0x00001000;
pub const DLT_TYPE_INFO_SCOD: u32 = 0x00038000;

pub const DLT_TYLE_8BIT: u32 = 0x00000001;
pub const DLT_TYLE_16BIT: u32 = 0x00000002;
pub const DLT_TYLE_32BIT: u32 = 0x00000003;
pub const DLT_TYLE_64BIT: u32 = 0x00000004;
pub const DLT_TYLE_128BIT: u32 = 0x00000005;

pub const DLT_SCOD_ASCII: u32 = 0x00000000;
pub const DLT_SCOD_UTF8: u32 = 0x00008000;

// ---------------------------------------------------------------------------
// Lookup tables
// ---------------------------------------------------------------------------

pub const MESSAGE_TYPE: [&str; 8] = ["log", "app_trace", "nw_trace", "control", "", "", "", ""];
pub const LOG_INFO: [&str; 16] = [
    "", "fatal", "error", "warn", "info", "debug", "verbose", "", "", "", "", "", "", "", "", "",
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
pub const DLT_SERVICE_ID_LAST_ENTRY: u8 = 0x15;
