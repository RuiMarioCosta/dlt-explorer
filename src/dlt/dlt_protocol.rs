// This file is meant to recreate the varriables, functions, macros, etc in dlt-daemon's
// dlt_protocol.h file so that it can be easier to update

/*
 * Definitions of the htyp parameter in standard header.
 */
pub const DLT_HTYP_UEH: u8 = 0x01; // use extended header
pub const _DLT_HTYP_MSBF: u8 = 0x02; // MSB first
pub const DLT_HTYP_WEID: u8 = 0x04; // with ECU ID
pub const DLT_HTYP_WSID: u8 = 0x08; // with session ID
pub const DLT_HTYP_WTMS: u8 = 0x10; // with timestamp
pub const _DLT_HTYP_VERS: u8 = 0xe0; // version number, 0x1

pub fn dlt_is_htyp_ueh(htyp: u8) -> bool {
    htyp & DLT_HTYP_UEH != 0
}

pub fn dlt_is_htyp_weid(htyp: u8) -> bool {
    htyp & DLT_HTYP_WEID != 0
}

pub fn dlt_is_htyp_wsid(htyp: u8) -> bool {
    htyp & DLT_HTYP_WSID != 0
}

pub fn dlt_is_htyp_wtms(htyp: u8) -> bool {
    htyp & DLT_HTYP_WTMS != 0
}

/*
 * Definitions of msin parameter in extended header.
 */
pub const DLT_MSIN_VERB: u8 = 0x01; // verbose
pub const DLT_MSIN_MSTP: u8 = 0x0e; // message type
pub const DLT_MSIN_MTIN: u8 = 0xf0; // message type info

pub const DLT_MSIN_MSTP_SHIFT: u8 = 1; // shift right offset to get mstp value
pub const DLT_MSIN_MTIN_SHIFT: u8 = 4; // shift right offset to get mtin value

pub fn dlt_is_msin_verb(msin: u8) -> bool {
    (msin & DLT_MSIN_VERB) != 0
}

pub fn dlt_get_msin_mstp(msin: u8) -> u8 {
    (msin & DLT_MSIN_MSTP) >> DLT_MSIN_MSTP_SHIFT
}

pub fn dlt_get_msin_mtin(msin: u8) -> u8 {
    (msin & DLT_MSIN_MTIN) >> DLT_MSIN_MTIN_SHIFT
}

/*
 * Definitions of mstp parameter in extended header.
 */
pub const _DLT_TYPE_LOG: u8 = 0x00; // Log message type
pub const _DLT_TYPE_APP_TRACE: u8 = 0x01; // Application trace message type
pub const _DLT_TYPE_NW_TRACE: u8 = 0x02; // Network trace message type
pub const DLT_TYPE_CONTROL: u8 = 0x03; // Control message type

/*
 * Definitions of msci parameter in extended header.
 */
pub const _DLT_CONTROL_REQUEST: u8 = 0x01; // Request message
pub const DLT_CONTROL_RESPONSE: u8 = 0x02; // Response to request message
pub const _DLT_CONTROL_TIME: u8 = 0x03; // keep-alive message

/*
 * Definitions of types of arguments in payload.
 */
pub const DLT_TYPE_INFO_TYLE: u32 = 0x0000000f; // Length of standard data: 1 = 8bit, 2 = 16bit, 3 = 32 bit, 4 = 64 bit, 5 = 128 bit
pub const DLT_TYPE_INFO_BOOL: u32 = 0x00000010; // Boolean data
pub const DLT_TYPE_INFO_SINT: u32 = 0x00000020; // Signed integer data
pub const DLT_TYPE_INFO_UINT: u32 = 0x00000040; // Unsigned integer data
pub const DLT_TYPE_INFO_FLOA: u32 = 0x00000080; // Float data
pub const _DLT_TYPE_INFO_ARAY: u32 = 0x00000100; // Array of standard types
pub const DLT_TYPE_INFO_STRG: u32 = 0x00000200; // String
pub const DLT_TYPE_INFO_RAWD: u32 = 0x00000400; // Raw data
pub const DLT_TYPE_INFO_VARI: u32 = 0x00000800; // Set, if additional information to a variable is available
pub const DLT_TYPE_INFO_FIXP: u32 = 0x00001000; // Set, if quantization and offset
pub const _DLT_TYPE_INFO_TRAI: u32 = 0x00002000; // Set, if additional trace information is added
pub const _DLT_TYPE_INFO_STRU: u32 = 0x00004000; // Struct
pub const DLT_TYPE_INFO_SCOD: u32 = 0x00038000; // coding of the type string: 0 = ASCII, 1 = UTF-8

pub const DLT_TYLE_8BIT: u32 = 0x00000001;
pub const DLT_TYLE_16BIT: u32 = 0x00000002;
pub const DLT_TYLE_32BIT: u32 = 0x00000003;
pub const DLT_TYLE_64BIT: u32 = 0x00000004;
pub const DLT_TYLE_128BIT: u32 = 0x00000005;

pub const DLT_SCOD_ASCII: u32 = 0x00000000;
pub const DLT_SCOD_UTF8: u32 = 0x00008000;
pub const _DLT_SCOD_HEX: u32 = 0x00010000;
pub const _DLT_SCOD_BIN: u32 = 0x00018000;

/*
 * Definitions of DLT services.
 */
// #define DLT_SERVICE_ID_CALLSW_CINJECTION 0xFFF
//
// enum dlt_services {
//     DLT_SERVICE_ID = 0x00,
//     DLT_SERVICE_ID_SET_LOG_LEVEL = 0x01,
//     DLT_SERVICE_ID_SET_TRACE_STATUS = 0x02,
//     DLT_SERVICE_ID_GET_LOG_INFO = 0x03,
//     DLT_SERVICE_ID_GET_DEFAULT_LOG_LEVEL = 0x04,
//     DLT_SERVICE_ID_STORE_CONFIG = 0x05,
//     DLT_SERVICE_ID_RESET_TO_FACTORY_DEFAULT = 0x06,
//     DLT_SERVICE_ID_SET_COM_INTERFACE_STATUS = 0x07,
//     DLT_SERVICE_ID_SET_COM_INTERFACE_MAX_BANDWIDTH = 0x08,
//     DLT_SERVICE_ID_SET_VERBOSE_MODE = 0x09,
//     DLT_SERVICE_ID_SET_MESSAGE_FILTERING = 0x0A,
//     DLT_SERVICE_ID_SET_TIMING_PACKETS = 0x0B,
//     DLT_SERVICE_ID_GET_LOCAL_TIME = 0x0C,
//     DLT_SERVICE_ID_USE_ECU_ID = 0x0D,
//     DLT_SERVICE_ID_USE_SESSION_ID = 0x0E,
//     DLT_SERVICE_ID_USE_TIMESTAMP = 0x0F,
//     DLT_SERVICE_ID_USE_EXTENDED_HEADER = 0x10,
//     DLT_SERVICE_ID_SET_DEFAULT_LOG_LEVEL = 0x11,
//     DLT_SERVICE_ID_SET_DEFAULT_TRACE_STATUS = 0x12,
//     DLT_SERVICE_ID_GET_SOFTWARE_VERSION = 0x13,
//     DLT_SERVICE_ID_MESSAGE_BUFFER_OVERFLOW = 0x14,
//     DLT_SERVICE_ID_LAST_ENTRY
// };
pub const DLT_SERVICE_ID_LAST_ENTRY: u8 = 0x15;
