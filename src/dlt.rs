use anyhow::Result;
use byteorder::{NativeEndian, ReadBytesExt};
use core::fmt;
use memchr::memmem::Finder;
use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

mod dlt_common;
mod dlt_protocol;

use dlt_common::*;
use dlt_protocol::*;

const DLT_DELIMITER: &[u8] = b"DLT\x01";
const BUFFER_SIZE: usize = 1 << 13; // 8 KiB
const DLT_AVG_MESSAGE_SIZE: usize = 32; // rought estimate for average message sizes

fn get_files_size(files: &[PathBuf]) -> u64 {
    files
        .iter()
        .fold(0, |acc, file| acc + file.metadata().unwrap().len())
}

#[derive(Debug)]
pub struct Dlt<'a> {
    paths: Vec<PathBuf>,
    filter: Option<PathBuf>,

    // Storage header
    seconds: Vec<u32>,
    microseconds: Vec<i32>,
    ecus: Vec<String>,
    // Standard header
    htyps: Vec<u8>,
    mcnts: Vec<u8>,
    lens: Vec<u16>,
    // Extra header
    seids: Vec<u32>,
    tmsps: Vec<u32>,
    // Extended header
    msins: Vec<u8>,
    noars: Vec<u8>,
    apids: Vec<String>,
    ctids: Vec<String>,
    // message metadata
    message_types: Vec<&'a str>,
    log_infos: Vec<&'a str>,
    service_id_names: Vec<Cow<'a, str>>,
    return_types: Vec<&'a str>,
    // payloads
    payloads: Vec<String>,

    size: usize,
}

impl<'a> Dlt<'a> {
    pub fn from_files(paths: Vec<PathBuf>, filter: Option<PathBuf>) -> Result<Self> {
        let number_of_rows_estimate = get_files_size(&paths) as usize / DLT_AVG_MESSAGE_SIZE;

        let mut seconds = Vec::with_capacity(number_of_rows_estimate);
        let mut microseconds = Vec::with_capacity(number_of_rows_estimate);
        let mut ecus = Vec::with_capacity(number_of_rows_estimate);
        let mut htyps = Vec::with_capacity(number_of_rows_estimate);
        let mut mcnts = Vec::with_capacity(number_of_rows_estimate);
        let mut lens = Vec::with_capacity(number_of_rows_estimate);
        let mut seids = Vec::with_capacity(number_of_rows_estimate);
        let mut tmsps = Vec::with_capacity(number_of_rows_estimate);
        let mut msins = Vec::with_capacity(number_of_rows_estimate);
        let mut noars = Vec::with_capacity(number_of_rows_estimate);
        let mut apids = Vec::with_capacity(number_of_rows_estimate);
        let mut ctids = Vec::with_capacity(number_of_rows_estimate);
        let mut message_types = Vec::with_capacity(number_of_rows_estimate);
        let mut log_infos = Vec::with_capacity(number_of_rows_estimate);
        let mut service_id_names = Vec::with_capacity(number_of_rows_estimate);
        let mut return_types = Vec::with_capacity(number_of_rows_estimate);
        let mut payloads = Vec::with_capacity(number_of_rows_estimate);
        let mut size = 0;

        for path in &paths {
            let file = File::open(path)?;
            let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
            let mut buf = [0; BUFFER_SIZE];
            let finder = Finder::new(DLT_DELIMITER);

            let mut length: usize;
            loop {
                length = reader.read(&mut buf)?;
                if length == 0 {
                    break;
                }

                let positions = if length < BUFFER_SIZE {
                    finder
                        .find_iter(&buf[..length])
                        .chain(std::iter::once(length))
                        .collect()
                } else {
                    let positions: Vec<usize> = finder.find_iter(&buf).collect();
                    let last = positions.last().unwrap();
                    reader.seek_relative(*last as i64 - BUFFER_SIZE as i64)?;
                    positions
                };

                for window in positions.windows(2) {
                    let begin = window[0];
                    let end = window[1];
                    let mut message = &buf[begin + DLT_ID_SIZE..end];

                    // Storage header
                    seconds.push(message.read_u32::<NativeEndian>()?);
                    microseconds.push(message.read_i32::<NativeEndian>()?);
                    let mut ecu = String::from_utf8_lossy(&message[..DLT_ID_SIZE]).to_string();
                    message = &message[DLT_ID_SIZE..];
                    // TODO: check storage header

                    // Standard header
                    let htyp = message.read_u8()?;
                    htyps.push(htyp);
                    mcnts.push(message.read_u8()?);
                    lens.push(message.read_u16::<NativeEndian>()?);

                    // Extra header
                    let mut seid: u32 = 0;
                    let mut tmsp: u32 = 0;
                    let extra_header_size = dlt_standard_header_extra_size(htyp);
                    if extra_header_size != 0 {
                        if dlt_is_htyp_weid(htyp) {
                            ecu = String::from_utf8_lossy(&message[..DLT_ID_SIZE]).to_string();
                            message = &message[DLT_ID_SIZE..];
                        }
                        if dlt_is_htyp_wsid(htyp) {
                            seid = message.read_u32::<NativeEndian>()?;
                        }
                        if dlt_is_htyp_wtms(htyp) {
                            tmsp = message.read_u32::<NativeEndian>()?;
                        }
                    }
                    ecus.push(ecu);
                    seids.push(seid);
                    tmsps.push(tmsp);

                    // Extended header
                    let mut msin = 0;
                    let mut noar = 0;
                    let mut apid = String::new();
                    let mut ctid = String::new();
                    let mut message_type = "";
                    let mut log_info = "";
                    if dlt_is_htyp_ueh(htyp) {
                        msin = message.read_u8()?;
                        noar = message.read_u8()?;
                        apid = String::from_utf8_lossy(&message[..DLT_ID_SIZE]).to_string();
                        message = &message[DLT_ID_SIZE..];
                        ctid = String::from_utf8_lossy(&message[..DLT_ID_SIZE]).to_string();
                        message = &message[DLT_ID_SIZE..];
                        message_type = MESSAGE_TYPE[dlt_get_msin_mstp(msin) as usize];
                        log_info = LOG_INFO[dlt_get_msin_mtin(msin) as usize];
                    }
                    msins.push(msin);
                    noars.push(noar);
                    apids.push(apid);
                    ctids.push(ctid);
                    message_types.push(message_type);
                    log_infos.push(log_info);

                    // Payload
                    let mut service_id_name = Cow::Borrowed("");
                    let mut return_type = "";
                    let mut payload = String::new();
                    if dlt_msg_is_nonverbose(htyp, msin) {
                        // non-verbose mode the payload buffer can be:
                        // | service id name | return type | payload |

                        let id = message.read_u32::<NativeEndian>()?;
                        // TODO: is this endian calculation needed?
                        // let _id_tmp = dlt_endian_get_32(htyp as u32, id);

                        if dlt_msg_is_control(htyp, msin) {
                            if id < DLT_SERVICE_ID_LAST_ENTRY as u32 {
                                service_id_name = Cow::Borrowed(SERVICE_ID_NAME[id as usize]);
                            } else {
                                service_id_name = Cow::Owned(format!("service({})", id));
                            }
                        } else {
                            write!(&mut payload, "{}", id)?;
                        }

                        if dlt_msg_is_control_response(htyp, msin) {
                            let retval = message.read_u8()?;
                            return_type = RETURN_TYPE[retval as usize];
                        }

                        // reserve space for service id name, hex bytes and spaces to avoid reallocation
                        payload.reserve(service_id_name.len() + 3 * message.len());
                        write!(&mut payload, "{}", service_id_name)?;
                        for byte in message.iter() {
                            write!(&mut payload, " {:02x}", byte)?;
                        }
                    } else {
                        /* At this point, it is ensured that a extended header is available */

                        // verbose mode the payload buffer can be:
                        // | type info | payload | [ type_info | payload | ...]

                        for n in 0..noar {
                            if n > 0 {
                                write!(&mut payload, " ")?;
                            }

                            let type_info = message.read_u32::<NativeEndian>()?;
                            // TODO: is this endian calculation needed?
                            // let _type_info_tmp = dlt_endian_get_32(htyp as u32, type_info);

                            if (type_info & DLT_TYPE_INFO_STRG != 0)
                                && (type_info & DLT_TYPE_INFO_SCOD == DLT_SCOD_ASCII
                                    || type_info & DLT_TYPE_INFO_SCOD == DLT_SCOD_UTF8)
                            {
                                let length = message.read_u16::<NativeEndian>()?;

                                if type_info & DLT_TYPE_INFO_VARI != 0 {
                                    panic!("DLT_TYPE_INFO_VARI not implemented");
                                }

                                payload.reserve(length as usize);
                                // TODO: check if write! is faster
                                write!(
                                    &mut payload,
                                    "{}",
                                    String::from_utf8_lossy(&message[..length as usize])
                                )?;
                                message = &message[length as usize..];
                            } else if type_info & DLT_TYPE_INFO_BOOL != 0 {
                                if type_info & DLT_TYPE_INFO_VARI != 0 {
                                    panic!("DLT_TYPE_INFO_VARI not implemented");
                                }

                                let value: bool = message.read_u8()? != 0;
                                write!(&mut payload, "{}", value)?;
                            } else if (type_info & DLT_TYPE_INFO_SINT != 0)
                                || (type_info & DLT_TYPE_INFO_UINT != 0)
                            {
                                if type_info & DLT_TYPE_INFO_VARI != 0 {
                                    panic!("DLT_TYPE_INFO_VARI not implemented");
                                }
                                if type_info & DLT_TYPE_INFO_FIXP != 0 {
                                    panic!("DLT_TYPE_INFO_FIXP not implemented");
                                }

                                match type_info & DLT_TYPE_INFO_TYLE {
                                    DLT_TYLE_8BIT => {
                                        if type_info & DLT_TYPE_INFO_SINT != 0 {
                                            let value = message.read_i8()?;
                                            write!(&mut payload, "{}", value)?;
                                        } else {
                                            let value = message.read_u8()?;
                                            write!(&mut payload, "{}", value)?;
                                        }
                                    }
                                    DLT_TYLE_16BIT => {
                                        if type_info & DLT_TYPE_INFO_SINT != 0 {
                                            let value = message.read_i16::<NativeEndian>()?;
                                            write!(&mut payload, "{}", value)?;
                                        } else {
                                            let value = message.read_u16::<NativeEndian>()?;
                                            write!(&mut payload, "{}", value)?;
                                        }
                                    }
                                    DLT_TYLE_32BIT => {
                                        if type_info & DLT_TYPE_INFO_SINT != 0 {
                                            let value = message.read_i32::<NativeEndian>()?;
                                            write!(&mut payload, "{}", value)?;
                                        } else {
                                            let value = message.read_u32::<NativeEndian>()?;
                                            write!(&mut payload, "{}", value)?;
                                        }
                                    }
                                    DLT_TYLE_64BIT => {
                                        if type_info & DLT_TYPE_INFO_SINT != 0 {
                                            let value = message.read_i64::<NativeEndian>()?;
                                            write!(&mut payload, "{}", value)?;
                                        } else {
                                            let value = message.read_u64::<NativeEndian>()?;
                                            write!(&mut payload, "{}", value)?;
                                        }
                                    }
                                    DLT_TYLE_128BIT => {
                                        if type_info & DLT_TYPE_INFO_SINT != 0 {
                                            let value = message.read_i128::<NativeEndian>()?;
                                            write!(&mut payload, "{}", value)?;
                                        } else {
                                            let value = message.read_u128::<NativeEndian>()?;
                                            write!(&mut payload, "{}", value)?;
                                        }
                                    }
                                    _ => panic!("Number size is  bigger than 128 bits"),
                                }
                            } else if type_info & DLT_TYPE_INFO_FLOA != 0 {
                                if type_info & DLT_TYPE_INFO_VARI != 0 {
                                    panic!("DLT_TYPE_INFO_VARI not implemented");
                                }

                                match type_info & DLT_TYPE_INFO_TYLE {
                                    DLT_TYLE_8BIT => {
                                        panic!("No float conversion for 8 bit number");
                                    }
                                    DLT_TYLE_16BIT => {
                                        panic!("No float conversion for 16 bit number");
                                    }
                                    DLT_TYLE_32BIT => {
                                        let value = message.read_f32::<NativeEndian>()?;
                                        write!(&mut payload, "{}", value)?;
                                    }
                                    DLT_TYLE_64BIT => {
                                        let value = message.read_f64::<NativeEndian>()?;
                                        write!(&mut payload, "{}", value)?;
                                    }
                                    DLT_TYLE_128BIT => {
                                        panic!("No float conversion for 128 bit number");
                                    }
                                    _ => panic!("Number size is  bigger than 128 bits"),
                                }
                            } else if type_info & DLT_TYPE_INFO_RAWD != 0 {
                                let length = message.read_u16::<NativeEndian>()?;

                                // reserve space for service id name, hex bytes and spaces to avoid reallocation
                                payload.reserve(3 * message.len());

                                let mut iter = message.iter();
                                let byte = iter.next().unwrap();
                                write!(&mut payload, "{:02x}", byte)?;
                                for byte in iter {
                                    write!(&mut payload, " {:02x}", byte)?;
                                }
                                message = &message[length as usize..];
                            }
                        }
                    }
                    service_id_names.push(service_id_name);
                    return_types.push(return_type);
                    payloads.push(payload);

                    size += 1;
                }
            }
        }

        // TODO: add asserts of sizes

        Ok(Self {
            paths,
            filter,
            seconds,
            microseconds,
            ecus,
            htyps,
            mcnts,
            lens,
            seids,
            tmsps,
            msins,
            noars,
            apids,
            ctids,
            message_types,
            log_infos,
            service_id_names,
            return_types,
            payloads,
            size,
        })
    }

    pub fn apids(&self) -> &[String] {
        &self.apids
    }

    pub fn ctids(&self) -> &[String] {
        &self.ctids
    }

    pub fn payloads(&self) -> &[String] {
        &self.payloads
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Display for Dlt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}, {:?}, {}", self.paths, self.filter, self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_dlt_control_messages() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from("tests/data/testfile_control_messages.dlt"));
        let paths = vec![path];

        let result = Dlt::from_files(paths, None).unwrap();

        assert_eq!(result.apids(), vec!["APP\0"; 4]);
        assert_eq!(result.ctids(), vec!["CON\0"; 4]);
        assert_eq!(
            result.payloads(),
            vec![
                "set_default_log_level 04 72 65 6d 6f",
                "set_default_trace_status 00 72 65 6d 6f",
                "set_verbose_mode 01",
                "set_timing_packets 00"
            ]
        );
        assert_eq!(result.size(), 4);
    }

    #[test]
    fn parse_dlt_empty_number_and_text_messages() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from(
            "tests/data/testfile_empty_number_and_text.dlt",
        ));
        let paths = vec![path];

        let result = Dlt::from_files(paths, None).unwrap();

        assert_eq!(result.apids(), vec!["LOG\0"; 3]);
        assert_eq!(result.ctids(), vec!["TES1"; 3]);
        assert_eq!(result.payloads(), vec!["", "1011", "Hello BMW\0"]);
        assert_eq!(result.size(), 3);
    }

    #[test]
    fn parse_dlt_single_payloads() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from("tests/data/testfile_single_payloads.dlt"));
        let paths = vec![path];

        let result = Dlt::from_files(paths, None).unwrap();

        assert_eq!(result.apids(), vec!["LOG\0"; 16]);
        assert_eq!(result.ctids(), vec!["TES2"; 16]);
        assert_eq!(
            result.payloads(),
            vec![
                "101",
                "102",
                "103",
                "104",
                "105",
                "106",
                "107",
                "108",
                "109",
                "110",
                "true",
                "STRING 112 message\0",
                "CSTRING 113 message\0",
                "1.1",
                "1.2",
                "48 65 6c 6c 6f 20 77 6f 72 6c 64 00"
            ]
        );
    }

    #[test]
    fn parse_dlt_multiple_number_of_arguments() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from(
            "tests/data/testfile_multiple_number_of_arguments.dlt",
        ));
        let paths = vec![path];

        let result = Dlt::from_files(paths, None).unwrap();

        assert_eq!(result.apids(), vec!["LOG\0"; 7]);
        assert_eq!(result.ctids(), vec!["TES3"; 7]);
        assert_eq!(
            result.payloads(),
            vec![
                "",
                "21",
                "31 32",
                "41 42 43",
                "51 52 53 54",
                "61 62 63 64 65",
                "71 72 73 74 75 76",
            ]
        );
    }

    #[test]
    fn parse_dlt_number_and_text() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from("tests/data/testfile_number_and_text.dlt"));
        let paths = vec![path];

        let result = Dlt::from_files(paths, None).unwrap();

        assert_eq!(result.apids(), vec!["LOG\0"; 18]);
        assert_eq!(result.ctids(), vec!["TES4"; 18]);
        assert_eq!(
            result.payloads(),
            vec![
                "0 Hello world\0",
                "1 Hello world\0",
                "2 Hello world\0",
                "3 Hello world\0",
                "4 Hello world\0",
                "5 Hello world\0",
                "6 Hello world\0",
                "7 Hello world\0",
                "8 Hello world\0",
                "9 Hello world\0",
                "10 Hello world\0",
                "11 Hello world\0",
                "12 Hello world\0",
                "13 Hello world\0",
                "14 Hello world\0",
                "15 Hello world\0",
                "16 Hello world\0",
                "17 Hello world\0",
            ]
        );
    }

    #[test]
    fn parse_dlt_type_id_and_text() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from("tests/data/testfile_type_id_and_text.dlt"));
        let paths = vec![path];

        let result = Dlt::from_files(paths, None).unwrap();

        assert_eq!(
            result.payloads(),
            vec![
                "set_default_log_level 04 72 65 6d 6f",
                "set_default_trace_status 00 72 65 6d 6f",
                "set_verbose_mode 01",
                "set_timing_packets 00",
                "101",
                "102 f3 03",
                "103 0a 00 48 65 6c 6c 6f 20 42 4d 57 00",
                "201 65",
                "202 66 00",
                "203 67 00 00 00",
                "204 68 00 00 00 00 00 00 00",
                "205 69",
                "206 6a 00",
                "207 6b 00 00 00",
                "208 6c 00 00 00 00 00 00 00",
                "209 6d 00 00 00",
                "210 6e 00 00 00",
                "211 6f",
                "212 13 00 53 54 52 49 4e 47 20 31 31 32 20 6d 65 73 73 61 67 65 00",
                "213 14 00 43 53 54 52 49 4e 47 20 31 31 33 20 6d 65 73 73 61 67 65 00",
                "214 cd cc 8c 3f",
                "215 33 33 33 33 33 33 f3 3f",
                "216 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "301",
                "302 15 00 00 00",
                "303 1f 00 00 00 20 00 00 00",
                "304 29 00 00 00 2a 00 00 00 2b 00 00 00",
                "305 33 00 00 00 34 00 00 00 35 00 00 00 36 00 00 00",
                "305 3d 00 00 00 3e 00 00 00 3f 00 00 00 40 00 00 00 41 00 00 00",
                "305 47 00 00 00 48 00 00 00 49 00 00 00 4a 00 00 00 4b 00 00 00 4c 00 00 00",
                "401 00 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 01 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 02 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 03 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 04 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 05 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 06 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 07 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 08 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 09 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 0a 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 0b 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 0c 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 0d 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 0e 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 0f 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 10 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 11 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 12 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 13 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 14 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 15 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 16 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 17 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 18 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 19 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
                "401 1a 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00",
            ]
        );
    }

    #[test]
    fn parse_dlt_with_size_bigger_than_buffer() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from("tests/data/testfile_100k_rows.dlt"));
        let paths = vec![path];

        let result = Dlt::from_files(paths, None).unwrap();

        assert_eq!(result.size(), 100_000);
    }

    #[test]
    fn parse_dlt_with_multiple_files() {
        let paths: Vec<PathBuf> = vec![
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR").to_string()
                    + "/tests/data/testfile_control_messages.dlt",
            ),
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR").to_string()
                    + "/tests/data/testfile_empty_number_and_text.dlt",
            ),
        ];

        let result = Dlt::from_files(paths, None).unwrap();

        assert_eq!(
            result.apids(),
            vec![
                "APP\0", "APP\0", "APP\0", "APP\0", "LOG\0", "LOG\0", "LOG\0"
            ]
        );
        assert_eq!(
            result.ctids(),
            vec!["CON\0", "CON\0", "CON\0", "CON\0", "TES1", "TES1", "TES1"]
        );
        assert_eq!(
            result.payloads(),
            vec![
                "set_default_log_level 04 72 65 6d 6f",
                "set_default_trace_status 00 72 65 6d 6f",
                "set_verbose_mode 01",
                "set_timing_packets 00",
                "",
                "1011",
                "Hello BMW\0"
            ]
        );
        assert_eq!(result.size(), 7);
    }

    #[test]
    fn get_files_size_with_one_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from("tests/data/testfile_control_messages.dlt"));
        let paths = vec![path];
        let expected = 180;

        let result = get_files_size(&paths);

        assert_eq!(result, expected);
    }

    #[test]
    fn get_files_size_with_files() {
        let paths: Vec<PathBuf> = vec![
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR").to_string()
                    + "/tests/data/testfile_control_messages.dlt",
            ),
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR").to_string() + "/tests/data/testfile_single_payloads.dlt",
            ),
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR").to_string()
                    + "/tests/data/testfile_empty_number_and_text.dlt",
            ),
        ];
        let expected = 180 + 112 + 652;

        let result = get_files_size(&paths);

        assert_eq!(result, expected);
    }
}
