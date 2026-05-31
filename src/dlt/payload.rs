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

// ---------------------------------------------------------------------------
// Shared payload decode functions
// ---------------------------------------------------------------------------

/// Helper to read a u16 from a slice with the given byte order.
#[inline]
fn read_u16(data: &[u8], big_endian: bool) -> u16 {
    let bytes: [u8; 2] = data[0..2].try_into().unwrap();
    if big_endian {
        u16::from_be_bytes(bytes)
    } else {
        u16::from_le_bytes(bytes)
    }
}

/// Helper to read a u32 from a slice with the given byte order.
#[inline]
fn read_u32(data: &[u8], big_endian: bool) -> u32 {
    let bytes: [u8; 4] = data[0..4].try_into().unwrap();
    if big_endian {
        u32::from_be_bytes(bytes)
    } else {
        u32::from_le_bytes(bytes)
    }
}

/// Helper to read a u64 from a slice with the given byte order.
#[inline]
fn read_u64(data: &[u8], big_endian: bool) -> u64 {
    let bytes: [u8; 8] = data[0..8].try_into().unwrap();
    if big_endian {
        u64::from_be_bytes(bytes)
    } else {
        u64::from_le_bytes(bytes)
    }
}

/// Helper to read an i16 from a slice with the given byte order.
#[inline]
fn read_i16(data: &[u8], big_endian: bool) -> i16 {
    let bytes: [u8; 2] = data[0..2].try_into().unwrap();
    if big_endian {
        i16::from_be_bytes(bytes)
    } else {
        i16::from_le_bytes(bytes)
    }
}

/// Helper to read an i32 from a slice with the given byte order.
#[inline]
fn read_i32(data: &[u8], big_endian: bool) -> i32 {
    let bytes: [u8; 4] = data[0..4].try_into().unwrap();
    if big_endian {
        i32::from_be_bytes(bytes)
    } else {
        i32::from_le_bytes(bytes)
    }
}

/// Helper to read an i64 from a slice with the given byte order.
#[inline]
fn read_i64(data: &[u8], big_endian: bool) -> i64 {
    let bytes: [u8; 8] = data[0..8].try_into().unwrap();
    if big_endian {
        i64::from_be_bytes(bytes)
    } else {
        i64::from_le_bytes(bytes)
    }
}

/// Helper to read an f32 from a slice with the given byte order.
#[inline]
fn read_f32(data: &[u8], big_endian: bool) -> f32 {
    let bytes: [u8; 4] = data[0..4].try_into().unwrap();
    if big_endian {
        f32::from_be_bytes(bytes)
    } else {
        f32::from_le_bytes(bytes)
    }
}

/// Helper to read an f64 from a slice with the given byte order.
#[inline]
fn read_f64(data: &[u8], big_endian: bool) -> f64 {
    let bytes: [u8; 8] = data[0..8].try_into().unwrap();
    if big_endian {
        f64::from_be_bytes(bytes)
    } else {
        f64::from_le_bytes(bytes)
    }
}

/// Decode a verbose payload into a human-readable string.
///
/// `big_endian` controls byte order for multi-byte reads (type_info, lengths, values).
pub fn decode_verbose(mut data: &[u8], big_endian: bool) -> String {
    let mut parts: Vec<String> = Vec::new();

    while data.len() >= 4 {
        let type_info = read_u32(data, big_endian);
        data = &data[4..];

        // Skip variable name if VARI is set
        if type_info & DLT_TYPE_INFO_VARI != 0 {
            if data.len() < 2 {
                break;
            }
            let name_len = read_u16(data, big_endian) as usize;
            data = &data[2..];
            if data.len() < name_len {
                break;
            }
            data = &data[name_len..];

            // For numeric types with VARI, there's also a unit name
            if type_info & (DLT_TYPE_INFO_SINT | DLT_TYPE_INFO_UINT | DLT_TYPE_INFO_FLOA) != 0 {
                if data.len() < 2 {
                    break;
                }
                let unit_len = read_u16(data, big_endian) as usize;
                data = &data[2..];
                if data.len() < unit_len {
                    break;
                }
                data = &data[unit_len..];
            }
        }

        let tyle = type_info & DLT_TYPE_INFO_TYLE;

        if type_info & DLT_TYPE_INFO_BOOL != 0 {
            if data.is_empty() {
                break;
            }
            let val = data[0] != 0;
            parts.push(if val { "true".into() } else { "false".into() });
            data = &data[1..];
        } else if type_info & DLT_TYPE_INFO_SINT != 0 {
            match tyle {
                DLT_TYLE_8BIT => {
                    if data.is_empty() {
                        break;
                    }
                    parts.push(format!("{}", data[0] as i8));
                    data = &data[1..];
                }
                DLT_TYLE_16BIT => {
                    if data.len() < 2 {
                        break;
                    }
                    let v = read_i16(data, big_endian);
                    parts.push(format!("{v}"));
                    data = &data[2..];
                }
                DLT_TYLE_32BIT => {
                    if data.len() < 4 {
                        break;
                    }
                    let v = read_i32(data, big_endian);
                    parts.push(format!("{v}"));
                    data = &data[4..];
                }
                DLT_TYLE_64BIT => {
                    if data.len() < 8 {
                        break;
                    }
                    let v = read_i64(data, big_endian);
                    parts.push(format!("{v}"));
                    data = &data[8..];
                }
                _ => break,
            }
        } else if type_info & DLT_TYPE_INFO_UINT != 0 {
            match tyle {
                DLT_TYLE_8BIT => {
                    if data.is_empty() {
                        break;
                    }
                    parts.push(format!("{}", data[0]));
                    data = &data[1..];
                }
                DLT_TYLE_16BIT => {
                    if data.len() < 2 {
                        break;
                    }
                    let v = read_u16(data, big_endian);
                    parts.push(format!("{v}"));
                    data = &data[2..];
                }
                DLT_TYLE_32BIT => {
                    if data.len() < 4 {
                        break;
                    }
                    let v = read_u32(data, big_endian);
                    parts.push(format!("{v}"));
                    data = &data[4..];
                }
                DLT_TYLE_64BIT => {
                    if data.len() < 8 {
                        break;
                    }
                    let v = read_u64(data, big_endian);
                    parts.push(format!("{v}"));
                    data = &data[8..];
                }
                _ => break,
            }
        } else if type_info & DLT_TYPE_INFO_FLOA != 0 {
            match tyle {
                DLT_TYLE_32BIT => {
                    if data.len() < 4 {
                        break;
                    }
                    let v = read_f32(data, big_endian);
                    parts.push(format!("{v}"));
                    data = &data[4..];
                }
                DLT_TYLE_64BIT => {
                    if data.len() < 8 {
                        break;
                    }
                    let v = read_f64(data, big_endian);
                    parts.push(format!("{v}"));
                    data = &data[8..];
                }
                _ => break,
            }
        } else if type_info & DLT_TYPE_INFO_STRG != 0 {
            if data.len() < 2 {
                break;
            }
            let str_len = read_u16(data, big_endian) as usize;
            data = &data[2..];
            if data.len() < str_len {
                break;
            }
            let str_data = &data[..str_len];
            // Strip null terminator if present
            let s = if str_data.last() == Some(&0) {
                &str_data[..str_len - 1]
            } else {
                str_data
            };
            let decoded = String::from_utf8_lossy(s).into_owned();
            parts.push(decoded);
            data = &data[str_len..];
        } else if type_info & DLT_TYPE_INFO_RAWD != 0 {
            if data.len() < 2 {
                break;
            }
            let raw_len = read_u16(data, big_endian) as usize;
            data = &data[2..];
            if data.len() < raw_len {
                break;
            }
            parts.push(hex_dump(&data[..raw_len]));
            data = &data[raw_len..];
        } else {
            // Unknown type — stop decoding, return partial result
            break;
        }
    }

    parts.join(" ")
}

/// Decode a non-verbose payload into a human-readable string.
///
/// `big_endian` controls byte order for the message ID read.
pub fn decode_non_verbose(data: &[u8], big_endian: bool) -> String {
    if data.len() < 4 {
        return format!("[non-verbose] {}", hex_dump(data));
    }
    let msg_id = read_u32(data, big_endian);
    let rest = &data[4..];
    format!("[non-verbose 0x{msg_id:08x}] {}", hex_dump(rest))
}

/// Decode a control payload into a human-readable string.
///
/// `big_endian` controls byte order for the service ID read.
pub fn decode_control(data: &[u8], big_endian: bool) -> String {
    if data.len() < 4 {
        return format!("[control] {}", hex_dump(data));
    }
    let service_id = read_u32(data, big_endian);
    let service_name = if (service_id as usize) < SERVICE_ID_NAME.len() {
        let name = SERVICE_ID_NAME[service_id as usize];
        if name.is_empty() {
            format!("0x{service_id:04x}")
        } else {
            name.to_string()
        }
    } else {
        format!("0x{service_id:04x}")
    };

    let rest = &data[4..];
    if rest.is_empty() {
        return format!("[control {service_name}]");
    }

    // First byte of response is the return type
    let return_type_id = rest[0] as usize;
    let return_text = if return_type_id < RETURN_TYPE.len() {
        let rt = RETURN_TYPE[return_type_id];
        if rt.is_empty() {
            hex_dump(rest)
        } else {
            rt.to_string()
        }
    } else {
        hex_dump(rest)
    };

    format!("[control {service_name}] {return_text}")
}

/// Format raw bytes as a space-separated hex dump.
pub fn hex_dump(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}
