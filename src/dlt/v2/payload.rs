use super::protocol::{CNTI_CONTROL, CNTI_NON_VERBOSE, CNTI_VERBOSE};
use crate::dlt::payload;

/// Decode a payload slice into a human-readable string based on content type.
///
/// v2 always uses big-endian byte order for payload fields.
pub fn decode_payload(cnti: u8, raw: &[u8]) -> String {
    match cnti {
        CNTI_VERBOSE => payload::decode_verbose(raw, true),
        CNTI_NON_VERBOSE => payload::decode_non_verbose(raw, true),
        CNTI_CONTROL => payload::decode_control(raw, true),
        _ => format!("[unknown cnti={cnti}] {}", payload::hex_dump(raw)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dlt::payload::*;

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
