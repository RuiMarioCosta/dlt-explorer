use super::protocol::{MESSAGE_TYPE_CONTROL, htyp_has_msbf, htyp_has_ueh, msin_is_verb, msin_mstp};
use crate::dlt::payload;

/// Decode a v1 payload slice into a human-readable string.
///
/// `htyp` is the v1 standard-header HTYP byte (carries UEH and MSBF flags).
/// `msin` is the raw MSIN byte from the extended header (0 when UEH is absent).
///
/// Dispatch rules:
/// - UEH + verbose bit  → `decode_verbose`
/// - UEH + control MSTP → `decode_control`
/// - otherwise          → `decode_non_verbose`
///
/// Byte order is determined by the MSBF flag in `htyp`.
pub fn decode_payload(htyp: u8, msin: u8, raw: &[u8]) -> String {
    let big_endian = htyp_has_msbf(htyp);

    if htyp_has_ueh(htyp) {
        if msin_is_verb(msin) {
            return payload::decode_verbose(raw, big_endian);
        }
        if msin_mstp(msin) == MESSAGE_TYPE_CONTROL {
            return payload::decode_control(raw, big_endian);
        }
    }

    payload::decode_non_verbose(raw, big_endian)
}
