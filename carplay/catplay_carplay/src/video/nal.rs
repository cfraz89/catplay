use exp_golomb::ExpGolombDecoder;

/// Represents NAL chunk in AnnexB or AVCC/HVCC-compatible buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NalChunk {
    pub prefix_start: usize,
    pub prefix_len: usize,
    pub data_size: usize,
}

#[derive(Debug)]
pub enum NalError {
    Ending,
    Underrun,
    Param,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum NalType {
    Unspecified(u8),
    NonIdrSlice,
    DataPartitionA,
    DataPartitionB,
    DataPartitionC,
    IdrSlice,
    Sei,
    Sps,
    Pps,
    Aud,
    EndOfSequence,
    EndOfStream,
    FillerData,
    SpsExt,
    PrefixNal,
    SubSps,
    Reserved(u8),
}

impl NalType {
    pub fn from_header(header: &[u8]) -> Self {
        NalType::from_byte(header[0])
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte & 0x1F {
            1 => NalType::NonIdrSlice,
            2 => NalType::DataPartitionA,
            3 => NalType::DataPartitionB,
            4 => NalType::DataPartitionC,
            5 => NalType::IdrSlice,
            6 => NalType::Sei,
            7 => NalType::Sps,
            8 => NalType::Pps,
            9 => NalType::Aud,
            10 => NalType::EndOfSequence,
            11 => NalType::EndOfStream,
            12 => NalType::FillerData,
            13 => NalType::SpsExt,
            14 => NalType::PrefixNal,
            15 => NalType::SubSps,
            r @ 16..=23 => NalType::Reserved(r),
            u => NalType::Unspecified(u),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            NalType::NonIdrSlice => 1,
            NalType::DataPartitionA => 2,
            NalType::DataPartitionB => 3,
            NalType::DataPartitionC => 4,
            NalType::IdrSlice => 5,
            NalType::Sei => 6,
            NalType::Sps => 7,
            NalType::Pps => 8,
            NalType::Aud => 9,
            NalType::EndOfSequence => 10,
            NalType::EndOfStream => 11,
            NalType::FillerData => 12,
            NalType::SpsExt => 13,
            NalType::PrefixNal => 14,
            NalType::SubSps => 15,
            NalType::Reserved(b) => *b,
            NalType::Unspecified(b) => *b,
        }
    }

    pub fn is_keyframe(&self) -> bool {
        matches!(self, NalType::IdrSlice)
    }
}

impl From<NalType> for u8 {
    fn from(val: NalType) -> Self {
        val.to_byte()
    }
}

impl From<u8> for NalType {
    fn from(value: u8) -> Self {
        Self::from_byte(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HevcNalType {
    TrailN,
    TrailR,
    TsaN,
    TsaR,
    StsaN,
    StsaR,
    RadlN,
    RadlR,
    RaslN,
    RaslR,
    RsvVcl(u8),
    BlaWLp,
    BlaWRadl,
    BlaNLp,
    IdrWRadl,
    IdrNLp,
    CraNut,
    RsvIrapVcl(u8),
    VpsNut,
    SpsNut,
    PpsNut,
    AudNut,
    EosNut,
    EobNut,
    FdNut,
    PrefixSeiNut,
    SuffixSeiNut,
    RsvNvcl(u8),
    Unspec(u8),
}

impl HevcNalType {
    /// Build from the first NAL header byte of HEVC bitstream.
    /// HEVC nal_unit_type occupies bits [1..6].
    pub fn from_byte(byte: u8) -> Self {
        let typ = (byte >> 1) & 0x3F;
        match typ {
            0 => HevcNalType::TrailN,
            1 => HevcNalType::TrailR,
            2 => HevcNalType::TsaN,
            3 => HevcNalType::TsaR,
            4 => HevcNalType::StsaN,
            5 => HevcNalType::StsaR,
            6 => HevcNalType::RadlN,
            7 => HevcNalType::RadlR,
            8 => HevcNalType::RaslN,
            9 => HevcNalType::RaslR,
            r @ 10..=15 => HevcNalType::RsvVcl(r),
            16 => HevcNalType::BlaWLp,
            17 => HevcNalType::BlaWRadl,
            18 => HevcNalType::BlaNLp,
            19 => HevcNalType::IdrWRadl,
            20 => HevcNalType::IdrNLp,
            21 => HevcNalType::CraNut,
            r @ 22..=31 => HevcNalType::RsvIrapVcl(r),
            32 => HevcNalType::VpsNut,
            33 => HevcNalType::SpsNut,
            34 => HevcNalType::PpsNut,
            35 => HevcNalType::AudNut,
            36 => HevcNalType::EosNut,
            37 => HevcNalType::EobNut,
            38 => HevcNalType::FdNut,
            39 => HevcNalType::PrefixSeiNut,
            40 => HevcNalType::SuffixSeiNut,
            r @ 41..=47 => HevcNalType::RsvNvcl(r),
            u => HevcNalType::Unspec(u),
        }
    }

    pub fn is_irap(self) -> bool {
        matches!(
            self,
            HevcNalType::BlaWLp
                | HevcNalType::BlaWRadl
                | HevcNalType::BlaNLp
                | HevcNalType::IdrWRadl
                | HevcNalType::IdrNLp
                | HevcNalType::CraNut
                | HevcNalType::RsvIrapVcl(_)
        )
    }
}

fn read_n_bits(reader: &mut ExpGolombDecoder, count: usize) -> Option<u8> {
    let mut value = 0u8;
    for _ in 0..count {
        value <<= 1;
        value |= reader.next_bit()?;
    }
    Some(value)
}

/// Decodes frame id used to detect the final NAL_NON_IDR piece to flush frame
pub fn get_frame_id(nal: &[u8]) -> Option<u64> {
    if nal.is_empty() || (nal[0] & 0x1F) != 1 {
        return None;
    }

    let slice = remove_emulation_prevention_bytes(&nal[1..]);
    let mut reader = ExpGolombDecoder::new(&slice, 0).unwrap();

    let _first_mb = reader.next_unsigned()?;
    let _slice_type = reader.next_unsigned()?; // ue(v)
    let _pps_id = reader.next_unsigned()?; // ue(v)

    let frame_num = read_n_bits(&mut reader, 4)? as u64;
    Some(frame_num)
}

/// Remove emulation prevention bytes (0x03 after 0x0000) from RBSP.
/// Example: 00 00 03 01 → 00 00 01
fn remove_emulation_prevention_bytes(nal: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(nal.len());
    let mut prev1 = 0u8;
    let mut prev2 = 0u8;

    for &b in nal {
        if prev1 == 0 && prev2 == 0 && b == 0x03 {
            continue;
        }
        out.push(b);
        prev2 = prev1;
        prev1 = b;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{HevcNalType, remove_emulation_prevention_bytes};

    #[test]
    fn test_remove_emulation_prevention_bytes() {
        let src = vec![0x00, 0x00, 0x01];
        let dst = remove_emulation_prevention_bytes(&src);
        assert_eq!(dst, src);

        let src = vec![0x00, 0x00, 0x03, 0x01];
        let dst = remove_emulation_prevention_bytes(&src);
        assert_eq!(dst, vec![0x00, 0x00, 0x01]);

        let src = vec![0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x02];
        let dst = remove_emulation_prevention_bytes(&src);
        assert_eq!(dst, vec![0x00, 0x00, 0x00, 0x00, 0x02]);

        let src = vec![0x12, 0x34, 0x00, 0x03, 0x56];
        let dst = remove_emulation_prevention_bytes(&src);
        assert_eq!(dst, src);

        let src: Vec<u8> = vec![];
        let dst = remove_emulation_prevention_bytes(&src);
        assert_eq!(dst, src);
    }

    #[test]
    fn test_hevc_nal_type_from_byte() {
        // type=32 (VPS): typ is in bits [1..6], so byte has to be (32 << 1)
        assert_eq!(HevcNalType::from_byte(32 << 1), HevcNalType::VpsNut);
        assert_eq!(HevcNalType::from_byte(33 << 1), HevcNalType::SpsNut);
        assert_eq!(HevcNalType::from_byte(34 << 1), HevcNalType::PpsNut);
        assert!(HevcNalType::from_byte(19 << 1).is_irap());
        assert!(!HevcNalType::from_byte(1 << 1).is_irap());
    }
}
