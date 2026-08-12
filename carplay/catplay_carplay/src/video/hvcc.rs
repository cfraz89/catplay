use log::debug;

use crate::video::{AvccConfig, HevcNalType};

const START_CODE: [u8; 4] = [0, 0, 0, 1];

#[derive(Debug, PartialEq, Clone)]
pub struct HvccConfig {
    pub nal_size_len: usize,
    pub vps_sps_pps: Vec<u8>,
}

/// The twelve profile_tier_level bytes of an HEVC SPS, laid out exactly as `hvcC` wants them,
/// with the sub-layer count that follows from the same header byte.
fn sps_profile_tier_level(sps: &[u8]) -> Option<([u8; 12], u8)> {
    // NAL header, then sps_video_parameter_set_id / sps_max_sub_layers_minus1 /
    // sps_temporal_id_nesting_flag, then the block itself.
    let rbsp = rbsp_unescape(sps, 15);
    let ptl: [u8; 12] = rbsp.get(3..15)?.try_into().ok()?;
    Some((ptl, ((rbsp[2] >> 1) & 0x07) + 1))
}

/// Drop the emulation-prevention bytes an encoder inserts into a NAL payload, up to `limit` bytes.
fn rbsp_unescape(nal: &[u8], limit: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(limit);
    let mut zeros = 0;
    for &byte in nal {
        if out.len() == limit {
            break;
        }
        if zeros >= 2 && byte == 0x03 {
            zeros = 0;
            continue;
        }
        zeros = if byte == 0 { zeros + 1 } else { 0 };
        out.push(byte);
    }
    out
}

/// Serialize AnnexB VPS/SPS/PPS (stored in [`AvccConfig::sps_pps`]) to HEVCDecoderConfigurationRecord (`hvcC`).
pub fn hvcc_config_serialize(avcc: &AvccConfig) -> Option<Vec<u8>> {
    if !(avcc.nal_size_len == 1 || avcc.nal_size_len == 2 || avcc.nal_size_len == 4) {
        return None;
    }

    let mut vps_list = Vec::new();
    let mut sps_list = Vec::new();
    let mut pps_list = Vec::new();

    let mut pos = 0;
    let mut nal_start = None;
    while pos + 3 < avcc.sps_pps.len() {
        if avcc.sps_pps[pos..].starts_with(&START_CODE) {
            if let Some(start) = nal_start {
                let nal = &avcc.sps_pps[start..pos];
                if !nal.is_empty() {
                    match HevcNalType::from_byte(nal[0]) {
                        HevcNalType::VpsNut => vps_list.push(nal.to_vec()),
                        HevcNalType::SpsNut => sps_list.push(nal.to_vec()),
                        HevcNalType::PpsNut => pps_list.push(nal.to_vec()),
                        _ => {}
                    }
                }
            }
            nal_start = Some(pos + 4);
            pos += 4;
        } else {
            pos += 1;
        }
    }
    if let Some(start) = nal_start {
        let nal = &avcc.sps_pps[start..];
        if !nal.is_empty() {
            match HevcNalType::from_byte(nal[0]) {
                HevcNalType::VpsNut => vps_list.push(nal.to_vec()),
                HevcNalType::SpsNut => sps_list.push(nal.to_vec()),
                HevcNalType::PpsNut => pps_list.push(nal.to_vec()),
                _ => {}
            }
        }
    }

    if sps_list.is_empty() || pps_list.is_empty() {
        return None;
    }

    let mut out = vec![
        0x01, // configurationVersion
        0x01, // profile_space/tier/profile_idc (minimal fallback)
        0x00,
        0x00,
        0x00,
        0x00, // profile_compatibility_flags
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // constraint flags
        0x00, // level_idc (unknown)
        0xF0,
        0x00, // reserved + min_spatial_segmentation_idc
        0xFC, // reserved + parallelismType
        0xFD, // reserved + chromaFormat (4:2:0 fallback)
        0xF8, // reserved + bitDepthLumaMinus8
        0xF8, // reserved + bitDepthChromaMinus8
        0x00,
        0x00,                                   // avgFrameRate
        0x04 | ((avcc.nal_size_len - 1) as u8), // temporalIdNested + lengthSizeMinusOne
        0x00,                                   // numOfArrays (filled below)
    ];

    // A receiver sets its decoder up from this record rather than from the parameter sets, so the
    // profile/tier/level has to be the stream's own - the skeleton above describes a stream at no
    // level and compatible with no profile, which is a decoder that never starts.
    if let Some((ptl, sub_layers)) = sps_profile_tier_level(&sps_list[0]) {
        out[1..13].copy_from_slice(&ptl);
        // constantFrameRate=0, then the sub-layer count. An iPhone leaves temporalIdNested clear
        // here even when its SPS sets the flag.
        out[21] = (sub_layers & 0x07) << 3 | ((avcc.nal_size_len - 1) as u8);
    }

    let mut num_arrays = 0u8;
    let mut append_array = |nal_type: u8, nals: &[Vec<u8>]| {
        if nals.is_empty() {
            return;
        }
        num_arrays = num_arrays.saturating_add(1);
        out.push(0x80 | nal_type); // array_completeness=1 + nal_unit_type
        out.extend_from_slice(&(nals.len() as u16).to_be_bytes());
        for nal in nals {
            out.extend_from_slice(&(nal.len() as u16).to_be_bytes());
            out.extend_from_slice(nal);
        }
    };

    append_array(32, &vps_list);
    append_array(33, &sps_list);
    append_array(34, &pps_list);
    out[22] = num_arrays;

    Some(out)
}

/// Extract codec configuration payload from a Visual Sample Entry (`avc1`/`hvc1`) blob.
///
/// Returns tuple `(atom_tag, atom_payload)` where tag is typically `avcC` or `hvcC`.
pub fn hvcc_sample_entry_extract_codec_config(data: &[u8]) -> Option<([u8; 4], Vec<u8>)> {
    const IMAGE_DESCRIPTION_BASE_LEN: usize = 86;
    if data.len() < IMAGE_DESCRIPTION_BASE_LEN + 8 {
        return None;
    }

    let id_size = u32::from_be_bytes(data[0..4].try_into().ok()?) as usize;
    let limit = id_size.min(data.len());
    if limit < IMAGE_DESCRIPTION_BASE_LEN + 8 {
        return None;
    }

    let mut pos = IMAGE_DESCRIPTION_BASE_LEN;
    while pos + 8 <= limit {
        let atom_size = u32::from_be_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
        if atom_size < 8 || pos + atom_size > limit {
            return None;
        }
        let atom_tag: [u8; 4] = data[pos + 4..pos + 8].try_into().ok()?;
        if &atom_tag == b"avcC" || &atom_tag == b"hvcC" {
            return Some((atom_tag, data[pos + 8..pos + atom_size].to_vec()));
        }
        pos += atom_size;
    }

    None
}

/// Deserialize HEVCDecoderConfigurationRecord (`hvcC`) into AnnexB parameter sets.
///
/// This is HEVC equivalent of `avcc_config_deserialize`.
pub fn hvcc_config_deserialize(data: &[u8]) -> Option<HvccConfig> {
    // Full fixed header up to and including numOfArrays.
    if data.len() < 23 || data[0] != 1 {
        return None;
    }

    // Byte 21: constantFrameRate(2) + numTemporalLayers(3) + temporalIdNested(1) + lengthSizeMinusOne(2)
    let nal_size_len = ((data[21] & 0x03) + 1) as usize;
    if !(nal_size_len == 1 || nal_size_len == 2 || nal_size_len == 4) {
        return None;
    }

    let num_of_arrays = data[22] as usize;
    let mut pos = 23;
    let mut header = Vec::new();

    for _ in 0..num_of_arrays {
        if pos + 3 > data.len() {
            return None;
        }
        // array header:
        // byte 0: array_completeness(1) + reserved(1) + nal_unit_type(6)
        let _nal_unit_type = data[pos] & 0x3f;
        pos += 1;
        let num_nalus = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        for _ in 0..num_nalus {
            if pos + 2 > data.len() {
                return None;
            }
            let len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            if pos + len > data.len() {
                return None;
            }
            header.extend_from_slice(&START_CODE);
            header.extend_from_slice(&data[pos..pos + len]);
            pos += len;
        }
    }

    debug!(
        "Parsed HVCC header: {:?}",
        HvccConfig {
            nal_size_len,
            vps_sps_pps: header.clone(),
        }
    );

    Some(HvccConfig {
        nal_size_len,
        vps_sps_pps: header,
    })
}

/// Builds a Visual Sample Entry atom (`stsd` payload item) from legacy codec config bytes
/// (`avcC`/`hvcC` body).
///
/// Output layout:
/// `[ImageDescriptionBase][child_atom_size:u32][child_atom_fourcc:u32][child_atom_payload...]`
pub fn hvcc_write_stsd_atom_from_old_format(width: u32, height: u32, config_payload: &[u8]) -> Vec<u8> {
    hvcc_write_stsd_atom_from_old_format_with_tags(width, height, config_payload, *b"avc1", *b"avcC")
}

/// Same as [`write_stsd_atom_from_old_format`] but allows choosing sample-entry/child atom tags,
/// e.g. (`hvc1`, `hvcC`) for HEVC.
pub fn hvcc_write_stsd_atom_from_old_format_with_tags(
    width: u32,
    height: u32,
    config_payload: &[u8],
    sample_entry_tag: [u8; 4],
    config_atom_tag: [u8; 4],
) -> Vec<u8> {
    const IMAGE_DESCRIPTION_BASE_LEN: usize = 86;
    // BT.709 primaries and matrix with the sRGB transfer, as an iPhone appends to every HEVC
    // config frame. Absent, a receiver has to guess how to interpret the picture.
    const COLR_ATOM: [u8; 18] = [
        0, 0, 0, 18, b'c', b'o', b'l', b'r', b'n', b'c', b'l', b'c', 0, 1, 0, 13, 0, 1,
    ];
    let colr: &[u8] = if sample_entry_tag == *b"hvc1" { &COLR_ATOM } else { &[] };
    let child_atom_size = (config_payload.len() + 8) as u32;
    let id_size = (IMAGE_DESCRIPTION_BASE_LEN + config_payload.len() + 8 + colr.len()) as u32;

    let mut out = Vec::with_capacity(id_size as usize);

    // ImageDescriptionBase
    out.extend_from_slice(&id_size.to_be_bytes()); // idSize
    out.extend_from_slice(&sample_entry_tag); // cType ('avc1'/'hvc1')
    out.extend_from_slice(&0u32.to_be_bytes()); // resvd1
    out.extend_from_slice(&0u16.to_be_bytes()); // resvd2
    // dataRefIndex and the two quality fields carry what an iPhone puts there; nothing in the
    // stream depends on them, and a receiver that checks them has one less reason to say no.
    out.extend_from_slice(&0xFFFFu16.to_be_bytes()); // dataRefIndex
    out.extend_from_slice(&0u16.to_be_bytes()); // version
    out.extend_from_slice(&0u16.to_be_bytes()); // revisionLevel
    out.extend_from_slice(&0u32.to_be_bytes()); // vendor
    out.extend_from_slice(&0x0000_0200u32.to_be_bytes()); // temporalQuality
    out.extend_from_slice(&0x0000_0200u32.to_be_bytes()); // spatialQuality
    out.extend_from_slice(&(width as u16).to_be_bytes()); // width
    out.extend_from_slice(&(height as u16).to_be_bytes()); // height
    out.extend_from_slice(&0x0048_0000u32.to_be_bytes()); // hRes (72 dpi)
    out.extend_from_slice(&0x0048_0000u32.to_be_bytes()); // vRes (72 dpi)
    out.extend_from_slice(&0u32.to_be_bytes()); // dataSize
    out.extend_from_slice(&1u16.to_be_bytes()); // frameCount

    // Pascal string, and it has to name the codec in the entry: an iPhone writes "HEVC" in an
    // `hvc1` entry. Writing the AVC name there described the stream as something it is not.
    let mut name = [0u8; 32];
    if sample_entry_tag == *b"hvc1" {
        name[0] = 4;
        name[1..5].copy_from_slice(b"HEVC");
    } else {
        name[0] = 6;
        name[1..7].copy_from_slice(b"'1cva'");
    }
    out.extend_from_slice(&name);

    out.extend_from_slice(&24u16.to_be_bytes()); // depth
    out.extend_from_slice(&0xFFFFu16.to_be_bytes()); // clutID (-1)

    // child atom: [size][fourcc][payload]
    out.extend_from_slice(&child_atom_size.to_be_bytes());
    out.extend_from_slice(&config_atom_tag);
    out.extend_from_slice(config_payload);
    out.extend_from_slice(colr);

    debug_assert_eq!(out.len(), id_size as usize);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::AvccConfig;

    #[test]
    fn test_write_stsd_atom_from_old_format_avcc() {
        let avcc = [1u8, 2, 3, 4, 5];
        let out = hvcc_write_stsd_atom_from_old_format(1920, 1080, &avcc);

        // idSize
        assert_eq!(u32::from_be_bytes(out[0..4].try_into().unwrap()) as usize, 86 + 8 + avcc.len());
        // cType
        assert_eq!(&out[4..8], b"avc1");
        // width/height offsets in ImageDescriptionBase
        assert_eq!(u16::from_be_bytes(out[32..34].try_into().unwrap()), 1920);
        assert_eq!(u16::from_be_bytes(out[34..36].try_into().unwrap()), 1080);
        // trailing atom
        assert_eq!(&out[86 + 4..86 + 8], b"avcC");
        assert_eq!(&out[86 + 8..], &avcc);
    }

    #[test]
    fn test_hvcc_config_deserialize() {
        // minimal-ish hvcC with 3 arrays: VPS, SPS, PPS
        let hvcc: Vec<u8> = vec![
            0x01, // configurationVersion
            0x01, // profile/tier/idc
            0, 0, 0, 0, // profile_compatibility_flags
            0, 0, 0, 0, 0, 0,    // constraint flags
            0x78, // level
            0xF0, 0x00, // min_spatial_segmentation_idc
            0xFC, // parallelismType
            0xFD, // chromaFormat
            0xF8, // bitDepthLumaMinus8
            0xF8, // bitDepthChromaMinus8
            0x00, 0x00, // avgFrameRate
            0xFF, // constantFrameRate/temporalIdNested/lengthSizeMinusOne => 4-byte NAL size
            0x03, // numOfArrays
            0x20, // VPS type(32)
            0x00, 0x01, // numNalus
            0x00, 0x02, 0x40, 0x01, // nal length + nal
            0x21, // SPS type(33)
            0x00, 0x01, // numNalus
            0x00, 0x03, 0x42, 0x01, 0x01, // nal length + nal
            0x22, // PPS type(34)
            0x00, 0x01, // numNalus
            0x00, 0x02, 0x44, 0x01, // nal length + nal
        ];

        let parsed = hvcc_config_deserialize(&hvcc).expect("hvcc should parse");
        assert_eq!(parsed.nal_size_len, 4);
        assert_eq!(
            parsed.vps_sps_pps,
            vec![
                0, 0, 0, 1, 0x40, 0x01, //
                0, 0, 0, 1, 0x42, 0x01, 0x01, //
                0, 0, 0, 1, 0x44, 0x01
            ]
        );
    }

    #[test]
    fn test_sample_entry_extract_codec_config_hvcc() {
        let hvcc = vec![1u8, 2, 3, 4];
        let atom = hvcc_write_stsd_atom_from_old_format_with_tags(1280, 720, &hvcc, *b"hvc1", *b"hvcC");
        let (tag, payload) = hvcc_sample_entry_extract_codec_config(&atom).expect("should extract");
        assert_eq!(&tag, b"hvcC");
        assert_eq!(payload, hvcc);
    }

    #[test]
    fn test_hvcc_config_serialize_roundtrip() {
        let cfg = AvccConfig {
            nal_size_len: 4,
            sps_pps: vec![
                0,
                0,
                0,
                1,
                32 << 1,
                0x01, // VPS
                0,
                0,
                0,
                1,
                33 << 1,
                0x02,
                0x03, // SPS
                0,
                0,
                0,
                1,
                34 << 1,
                0x04, // PPS
            ],
        };

        let hvcc = hvcc_config_serialize(&cfg).expect("hvcc should serialize");
        let parsed = hvcc_config_deserialize(&hvcc).expect("hvcc should parse");
        assert_eq!(parsed.nal_size_len, 4);
        assert_eq!(parsed.vps_sps_pps, cfg.sps_pps);
    }
}
