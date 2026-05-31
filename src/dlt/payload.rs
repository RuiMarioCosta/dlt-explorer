use super::dlt_common::{RETURN_TYPE, SERVICE_ID_NAME};
use super::dlt_protocol::*;
use super::protocol::{CNTI_CONTROL, CNTI_NON_VERBOSE, CNTI_VERBOSE};

/// Decode a payload slice into a human-readable string based on content type.
pub fn decode_payload(cnti: u8, raw: &[u8]) -> String {
    match cnti {
        CNTI_VERBOSE => decode_verbose(raw),
        CNTI_NON_VERBOSE => decode_non_verbose(raw),
        CNTI_CONTROL => decode_control(raw),
        _ => format!("[unknown cnti={cnti}] {}", hex_dump(raw)),
    }
}

fn decode_verbose(mut data: &[u8]) -> String {
    let mut parts: Vec<String> = Vec::new();

    while data.len() >= 4 {
        let type_info = u32::from_be_bytes(data[0..4].try_into().unwrap());
        data = &data[4..];

        // Skip variable name if VARI is set
        if type_info & DLT_TYPE_INFO_VARI != 0 {
            if data.len() < 2 {
                break;
            }
            let name_len = u16::from_be_bytes(data[0..2].try_into().unwrap()) as usize;
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
                let unit_len = u16::from_be_bytes(data[0..2].try_into().unwrap()) as usize;
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
                    let v = i16::from_be_bytes(data[0..2].try_into().unwrap());
                    parts.push(format!("{v}"));
                    data = &data[2..];
                }
                DLT_TYLE_32BIT => {
                    if data.len() < 4 {
                        break;
                    }
                    let v = i32::from_be_bytes(data[0..4].try_into().unwrap());
                    parts.push(format!("{v}"));
                    data = &data[4..];
                }
                DLT_TYLE_64BIT => {
                    if data.len() < 8 {
                        break;
                    }
                    let v = i64::from_be_bytes(data[0..8].try_into().unwrap());
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
                    let v = u16::from_be_bytes(data[0..2].try_into().unwrap());
                    parts.push(format!("{v}"));
                    data = &data[2..];
                }
                DLT_TYLE_32BIT => {
                    if data.len() < 4 {
                        break;
                    }
                    let v = u32::from_be_bytes(data[0..4].try_into().unwrap());
                    parts.push(format!("{v}"));
                    data = &data[4..];
                }
                DLT_TYLE_64BIT => {
                    if data.len() < 8 {
                        break;
                    }
                    let v = u64::from_be_bytes(data[0..8].try_into().unwrap());
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
                    let v = f32::from_be_bytes(data[0..4].try_into().unwrap());
                    parts.push(format!("{v}"));
                    data = &data[4..];
                }
                DLT_TYLE_64BIT => {
                    if data.len() < 8 {
                        break;
                    }
                    let v = f64::from_be_bytes(data[0..8].try_into().unwrap());
                    parts.push(format!("{v}"));
                    data = &data[8..];
                }
                _ => break,
            }
        } else if type_info & DLT_TYPE_INFO_STRG != 0 {
            if data.len() < 2 {
                break;
            }
            let str_len = u16::from_be_bytes(data[0..2].try_into().unwrap()) as usize;
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
            let raw_len = u16::from_be_bytes(data[0..2].try_into().unwrap()) as usize;
            data = &data[2..];
            if data.len() < raw_len {
                break;
            }
            parts.push(hex_dump(&data[..raw_len]));
            data = &data[raw_len..];
        } else {
            // Unknown type — stop decoding
            break;
        }
    }

    parts.join(" ")
}

fn decode_non_verbose(data: &[u8]) -> String {
    if data.len() < 4 {
        return format!("[non-verbose] {}", hex_dump(data));
    }
    let msg_id = u32::from_be_bytes(data[0..4].try_into().unwrap());
    let rest = &data[4..];
    format!("[non-verbose 0x{msg_id:08x}] {}", hex_dump(rest))
}

fn decode_control(data: &[u8]) -> String {
    if data.len() < 4 {
        return format!("[control] {}", hex_dump(data));
    }
    let service_id = u32::from_be_bytes(data[0..4].try_into().unwrap());
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

fn hex_dump(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to build a verbose TypeInfo + data
    fn make_bool(val: bool) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_BOOL | DLT_TYLE_8BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.push(if val { 1 } else { 0 });
        buf
    }

    fn make_sint8(val: i8) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_SINT | DLT_TYLE_8BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.push(val as u8);
        buf
    }

    fn make_sint16(val: i16) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_SINT | DLT_TYLE_16BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.extend_from_slice(&val.to_be_bytes());
        buf
    }

    fn make_sint32(val: i32) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_SINT | DLT_TYLE_32BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.extend_from_slice(&val.to_be_bytes());
        buf
    }

    fn make_sint64(val: i64) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_SINT | DLT_TYLE_64BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.extend_from_slice(&val.to_be_bytes());
        buf
    }

    fn make_uint8(val: u8) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_UINT | DLT_TYLE_8BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.push(val);
        buf
    }

    fn make_uint16(val: u16) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_UINT | DLT_TYLE_16BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.extend_from_slice(&val.to_be_bytes());
        buf
    }

    fn make_uint32(val: u32) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_UINT | DLT_TYLE_32BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.extend_from_slice(&val.to_be_bytes());
        buf
    }

    fn make_uint64(val: u64) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_UINT | DLT_TYLE_64BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.extend_from_slice(&val.to_be_bytes());
        buf
    }

    fn make_floa32(val: f32) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_FLOA | DLT_TYLE_32BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.extend_from_slice(&val.to_be_bytes());
        buf
    }

    fn make_floa64(val: f64) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_FLOA | DLT_TYLE_64BIT;
        let mut buf = type_info.to_be_bytes().to_vec();
        buf.extend_from_slice(&val.to_be_bytes());
        buf
    }

    fn make_strg_utf8(s: &str) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_STRG | DLT_SCOD_UTF8;
        let mut buf = type_info.to_be_bytes().to_vec();
        let len = (s.len() + 1) as u16; // +1 for null terminator
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(s.as_bytes());
        buf.push(0); // null terminator
        buf
    }

    fn make_strg_ascii(s: &str) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_STRG | DLT_SCOD_ASCII;
        let mut buf = type_info.to_be_bytes().to_vec();
        let len = (s.len() + 1) as u16;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(s.as_bytes());
        buf.push(0);
        buf
    }

    fn make_rawd(data: &[u8]) -> Vec<u8> {
        let type_info: u32 = DLT_TYPE_INFO_RAWD;
        let mut buf = type_info.to_be_bytes().to_vec();
        let len = data.len() as u16;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(data);
        buf
    }

    #[test]
    fn verbose_bool_true() {
        let payload = make_bool(true);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "true");
    }

    #[test]
    fn verbose_bool_false() {
        let payload = make_bool(false);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "false");
    }

    #[test]
    fn verbose_sint8() {
        let payload = make_sint8(-42);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "-42");
    }

    #[test]
    fn verbose_sint16() {
        let payload = make_sint16(-1000);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "-1000");
    }

    #[test]
    fn verbose_sint32() {
        let payload = make_sint32(-100_000);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "-100000");
    }

    #[test]
    fn verbose_sint64() {
        let payload = make_sint64(-9_000_000_000);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "-9000000000");
    }

    #[test]
    fn verbose_uint8() {
        let payload = make_uint8(255);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "255");
    }

    #[test]
    fn verbose_uint16() {
        let payload = make_uint16(65535);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "65535");
    }

    #[test]
    fn verbose_uint32() {
        let payload = make_uint32(4_000_000_000);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "4000000000");
    }

    #[test]
    fn verbose_uint64() {
        let payload = make_uint64(18_000_000_000_000);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "18000000000000");
    }

    #[test]
    fn verbose_floa32() {
        let payload = make_floa32(3.14);
        let result = decode_payload(CNTI_VERBOSE, &payload);
        assert!(result.starts_with("3.14"), "got: {result}");
    }

    #[test]
    fn verbose_floa64() {
        let payload = make_floa64(2.718281828);
        let result = decode_payload(CNTI_VERBOSE, &payload);
        assert!(result.starts_with("2.718281828"), "got: {result}");
    }

    #[test]
    fn verbose_strg_utf8() {
        let payload = make_strg_utf8("hello world");
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "hello world");
    }

    #[test]
    fn verbose_strg_ascii() {
        let payload = make_strg_ascii("test message");
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "test message");
    }

    #[test]
    fn verbose_rawd() {
        let payload = make_rawd(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(decode_payload(CNTI_VERBOSE, &payload), "de ad be ef");
    }

    #[test]
    fn verbose_multiple_arguments() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&make_uint32(42));
        payload.extend_from_slice(&make_strg_utf8("hello"));
        payload.extend_from_slice(&make_floa32(3.14));
        let result = decode_payload(CNTI_VERBOSE, &payload);
        let parts: Vec<&str> = result.split(' ').collect();
        assert_eq!(parts[0], "42");
        assert_eq!(parts[1], "hello");
        assert!(parts[2].starts_with("3.14"), "got: {}", parts[2]);
    }

    #[test]
    fn non_verbose_payload() {
        let mut payload = Vec::new();
        let msg_id: u32 = 0x0000_1234;
        payload.extend_from_slice(&msg_id.to_be_bytes());
        payload.extend_from_slice(&[0xAB, 0xCD]);
        let result = decode_payload(CNTI_NON_VERBOSE, &payload);
        assert_eq!(result, "[non-verbose 0x00001234] ab cd");
    }

    #[test]
    fn control_payload() {
        let mut payload = Vec::new();
        let service_id: u32 = 1; // set_log_level
        payload.extend_from_slice(&service_id.to_be_bytes());
        payload.push(0); // return type: ok
        let result = decode_payload(CNTI_CONTROL, &payload);
        assert_eq!(result, "[control set_log_level] ok");
    }
}
