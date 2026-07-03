#![no_std]
#![allow(unused)]

mod knob;
mod media_buttons;
mod proximity;
mod telephony;
mod touch_single;
mod touchpad;

pub use knob::*;
pub use media_buttons::*;
pub use proximity::*;
pub use telephony::*;
pub use touch_single::*;
pub use touchpad::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knob_reports_match_fill_report_layout() {
        assert_eq!(KnobReport::new(true, false, true, -1, 2, -3).to_bytes(), [0x05, 0xff, 0x02, 0xfd]);
        assert_eq!(KnobBasicReport::new(true, true, false, -2).to_bytes(), [0x03, 0xfe]);
        assert_eq!(KnobMinimalReport::new(true, -4).to_bytes(), [0x01, 0xfc]);
    }

    #[test]
    fn proximity_and_button_reports_are_single_byte() {
        assert_eq!(ProximityReport::new(true).to_bytes(), [1]);
        assert_eq!(MediaButtonsReport::new(MediaButton::NextTrack).to_bytes(), [4]);
        assert_eq!(TelephonyButtonsReport::new(TelephonyButton::Mute).to_bytes(), [4]);
        assert_eq!(TouchpadButtonsReport::new(true, false, true).to_bytes(), [0x05]);
    }

    #[test]
    fn descriptors_patch_dynamic_dimensions() {
        let touch = TouchScreenSingleReport::descriptor(0x1234, 0xabcd);
        assert_eq!(&touch[0x27..0x29], &[0x34, 0x12]);
        assert_eq!(&touch[0x34..0x36], &[0xcd, 0xab]);

        let touch_cancel = TouchScreenSingleCancelReport::descriptor(0x1234, 0xabcd);
        assert_eq!(&touch_cancel[0x2B..0x2D], &[0x34, 0x12]);
        assert_eq!(&touch_cancel[0x38..0x3A], &[0xcd, 0xab]);

        let touchpad = TouchpadOnlyReport::descriptor(0x1234, 0xabcd, 0x0201, 0x0403);
        assert_eq!(&touchpad[39..41], &[0x34, 0x12]);
        assert_eq!(&touchpad[44..46], &[0x01, 0x02]);
        assert_eq!(&touchpad[61..63], &[0xcd, 0xab]);
        assert_eq!(&touchpad[66..68], &[0x03, 0x04]);
    }

    #[test]
    fn touchpad_multi_character_report_pads_candidates() {
        let report = TouchpadMultiCharacterReport::new(2, *b"abzz", 90, 5, *b"wxyz", 80, 1, 0x1234, 0xabcd);

        assert_eq!(
            report.to_bytes(),
            [
                0x01, 0x34, 0x12, 0xcd, 0xab, b'a', b'b', 0x00, 0x00, 0x01, 0x02, 0x5a, b'w', b'x', b'y', b'z', 0x01, 0x04, 0x50,
            ]
        );
    }
}
