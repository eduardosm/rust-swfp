use swfp::{Float, FloatConvertFrom, FpStatus};

use crate::{ALL_ROUND_MODES, Loss, mk_f8e4m3b8nnz, mk_f8e4m3nao, mk_f8e5m2, mk_f16, mk_f32};

fn check_convert_round<F1, F2>(
    orig_value: F1,
    expected_conv_value_tz: F2,
    expected_conv_value_az: F2,
    sign: bool,
    loss: Loss,
) where
    F1: Float<Bits: std::fmt::Debug> + FloatConvertFrom<F2>,
    F2: Float<Bits: std::fmt::Debug> + FloatConvertFrom<F1>,
{
    super::check_float_round(
        |round| F2::convert_from_ex(orig_value, round),
        expected_conv_value_tz,
        expected_conv_value_az,
        sign,
        loss,
    );
}

fn check_convert_lossless<F1, F2>(orig_value: F1, expected_conv_value: F2)
where
    F1: Float<Bits: std::fmt::Debug> + FloatConvertFrom<F2>,
    F2: Float<Bits: std::fmt::Debug> + FloatConvertFrom<F1>,
{
    for round1 in ALL_ROUND_MODES {
        let (conv_value, status1) = F2::convert_from_ex(orig_value, round1);
        assert_eq!(status1, FpStatus::Ok);
        assert_eq!(conv_value.to_bits(), expected_conv_value.to_bits());

        for round2 in ALL_ROUND_MODES {
            let (new_value, status2) = F1::convert_from_ex(conv_value, round2);
            assert_eq!(status2, FpStatus::Ok);
            assert_eq!(new_value.to_bits(), orig_value.to_bits());
        }
    }
}

fn check_convert_snan<F1, F2>(orig_value: F1, expected_conv_value: F2)
where
    F1: Float<Bits: std::fmt::Debug> + FloatConvertFrom<F2>,
    F2: Float<Bits: std::fmt::Debug> + FloatConvertFrom<F1>,
{
    for round1 in ALL_ROUND_MODES {
        let (conv_value, status1) = F2::convert_from_ex(orig_value, round1);
        assert_eq!(status1, FpStatus::Invalid);
        assert_eq!(conv_value.to_bits(), expected_conv_value.to_bits());

        for round2 in ALL_ROUND_MODES {
            let (_, status2) = F1::convert_from_ex(conv_value, round2);
            assert_eq!(status2, FpStatus::Ok);
        }
    }
}

#[test]
fn test_convert_f16_to_f32() {
    for s in [false, true] {
        check_convert_lossless(mk_f16(s, -15, 0), mk_f32(s, -127, 0));

        for m in 1u16..(1 << 10) {
            let shift = m.leading_zeros() - 5;
            check_convert_lossless(
                mk_f16(s, -15, m),
                mk_f32(s, -14 - shift as i16, u32::from((m << shift) & 0x3FF) << 13),
            );
        }

        for e in -14..=15 {
            for m in 0..(1 << 10) {
                check_convert_lossless(
                    mk_f16(s, e, m),
                    mk_f32(s, i16::from(e), u32::from(m) << 13),
                );
            }
        }

        check_convert_lossless(mk_f16(s, 16, 0), mk_f32(s, 128, 0));

        for m in 1..(1 << 10) {
            if m & (1 << 9) != 0 {
                check_convert_lossless(mk_f16(s, 16, m), mk_f32(s, 128, u32::from(m) | (1 << 22)));
            } else {
                check_convert_snan(mk_f16(s, 16, m), mk_f32(s, 128, u32::from(m) | (1 << 22)));
            }
        }
    }
}

#[test]
fn test_convert_f32_to_f16() {
    fn mk_mants(hi: u16, lo: u32) -> (u32, u16) {
        assert!(hi < 1 << 10);
        assert!(lo < 1 << (23 - 10));
        let m32 = (u32::from(hi) << (23 - 10)) | lo;
        let m16 = hi;
        (m32, m16)
    }

    for s in [false, true] {
        let (m32, m16) = mk_mants(0x233, 0);
        check_convert_lossless(mk_f32(false, 0, m32), mk_f16(false, 0, m16));
        check_convert_lossless(mk_f32(true, 0, m32), mk_f16(true, 0, m16));

        let (m32, m16) = mk_mants(0x233, 0x130F);
        check_convert_round(
            mk_f32(s, 0, m32),
            mk_f16(s, 0, m16),
            mk_f16(s, 0, m16 + 1),
            s,
            Loss::HalfUp,
        );

        let (m32, m16) = mk_mants(0x233, 0x030F);
        check_convert_round(
            mk_f32(s, 0, m32),
            mk_f16(s, 0, m16),
            mk_f16(s, 0, m16 + 1),
            s,
            Loss::HalfDown,
        );

        let (m32, m16) = mk_mants(0x233, 0x1000);
        check_convert_round(
            mk_f32(s, 0, m32),
            mk_f16(s, 0, m16),
            mk_f16(s, 0, m16 + 1),
            s,
            Loss::HalfOdd,
        );

        let (m32, m16) = mk_mants(0x234, 0x1000);
        check_convert_round(
            mk_f32(s, 0, m32),
            mk_f16(s, 0, m16),
            mk_f16(s, 0, m16 + 1),
            s,
            Loss::HalfEven,
        );

        let (m32, m16) = mk_mants(0x3FF, 0x1FA0);
        check_convert_round(
            mk_f32(s, 0, m32),
            mk_f16(s, 0, m16),
            mk_f16(s, 1, 0),
            s,
            Loss::HalfUp,
        );

        let (m32, m16) = mk_mants(0x3FF, 0x0FA0);
        check_convert_round(
            mk_f32(s, 0, m32),
            mk_f16(s, 0, m16),
            mk_f16(s, 1, 0),
            s,
            Loss::HalfDown,
        );

        let (m32, m16) = mk_mants(0x3FF, 0x1000);
        check_convert_round(
            mk_f32(s, 0, m32),
            mk_f16(s, 0, m16),
            mk_f16(s, 1, 0),
            s,
            Loss::HalfOdd,
        );

        let (m32, m16) = mk_mants(0x230, 0x0000);
        let m16 = (m16 >> 4) | (1 << 6);
        check_convert_lossless(mk_f32(false, -18, m32), mk_f16(false, -15, m16));
        check_convert_lossless(mk_f32(true, -18, m32), mk_f16(true, -15, m16));

        for lo in [0x1FA0, 0x0FA0, 0x1000] {
            let (m32, m16) = mk_mants(0x230, lo);
            let m16 = (m16 >> 4) | (1 << 6);
            check_convert_round(
                mk_f32(s, -18, m32),
                mk_f16(s, -15, m16),
                mk_f16(s, -15, m16 + 1),
                s,
                Loss::HalfDown,
            );

            let (m32, m16) = mk_mants(0x238, lo);
            let m16 = (m16 >> 4) | (1 << 6);
            check_convert_round(
                mk_f32(s, -18, m32),
                mk_f16(s, -15, m16),
                mk_f16(s, -15, m16 + 1),
                s,
                Loss::HalfUp,
            );
        }

        for lo in [0x0000, 0x1FA0, 0x0FA0, 0x1000] {
            let (m32, m16) = mk_mants(0x239, lo);
            let m16 = (m16 >> 4) | (1 << 6);
            check_convert_round(
                mk_f32(s, -18, m32),
                mk_f16(s, -15, m16),
                mk_f16(s, -15, m16 + 1),
                s,
                Loss::HalfUp,
            );

            let (m32, m16) = mk_mants(0x232, lo);
            let m16 = (m16 >> 4) | (1 << 6);
            check_convert_round(
                mk_f32(s, -18, m32),
                mk_f16(s, -15, m16),
                mk_f16(s, -15, m16 + 1),
                s,
                Loss::HalfDown,
            );
        }

        let (m32, m16) = mk_mants(0x238, 0x0000);
        let m16 = (m16 >> 4) | (1 << 6);
        check_convert_round(
            mk_f32(s, -18, m32),
            mk_f16(s, -15, m16),
            mk_f16(s, -15, m16 + 1),
            s,
            Loss::HalfOdd,
        );

        let (m32, m16) = mk_mants(0x248, 0x0000);
        let m16 = (m16 >> 4) | (1 << 6);
        check_convert_round(
            mk_f32(s, -18, m32),
            mk_f16(s, -15, m16),
            mk_f16(s, -15, m16 + 1),
            s,
            Loss::HalfEven,
        );

        check_convert_round(
            mk_f32(s, -25, 0),
            mk_f16(s, -15, 0),
            mk_f16(s, -15, 1),
            s,
            Loss::HalfEven,
        );

        check_convert_round(
            mk_f32(s, -25, 1),
            mk_f16(s, -15, 0),
            mk_f16(s, -15, 1),
            s,
            Loss::HalfUp,
        );

        check_convert_round(
            mk_f32(s, -25, 1 << 20),
            mk_f16(s, -15, 0),
            mk_f16(s, -15, 1),
            s,
            Loss::HalfUp,
        );

        check_convert_round(
            mk_f32(s, -26, 0),
            mk_f16(s, -15, 0),
            mk_f16(s, -15, 1),
            s,
            Loss::HalfDown,
        );

        for m in 1..(1 << 23) {
            check_convert_round(
                mk_f32(s, -127, m),
                mk_f16(s, -15, 0),
                mk_f16(s, -15, 1),
                s,
                Loss::HalfDown,
            );
        }

        check_convert_round(
            mk_f32(s, 16, 0),
            mk_f16(s, 15, 0x3FF),
            mk_f16(s, 16, 0),
            s,
            Loss::Overflow,
        );
    }
}

#[test]
fn test_convert_f8e5m2_to_f16() {
    for s in [false, true] {
        check_convert_lossless(mk_f8e5m2(s, -15, 0), mk_f16(s, -15, 0));

        for m in 1u8..=0b11 {
            check_convert_lossless(mk_f8e5m2(s, -15, m), mk_f16(s, -15, u16::from(m) << 8));
        }

        for e in -14..=15 {
            for m in 0..=0b11 {
                check_convert_lossless(mk_f8e5m2(s, e, m), mk_f16(s, e, u16::from(m) << 8));
            }
        }

        check_convert_lossless(mk_f8e5m2(s, 16, 0), mk_f16(s, 16, 0));

        for m in 1..=0b11 {
            if m & 0b10 != 0 {
                check_convert_lossless(mk_f8e5m2(s, 16, m), mk_f16(s, 16, u16::from(m) | (1 << 9)));
            } else {
                check_convert_snan(mk_f8e5m2(s, 16, m), mk_f16(s, 16, u16::from(m) | (1 << 9)));
            }
        }
    }
}

#[test]
fn test_convert_f8e4m3b8nnz_to_f16() {
    check_convert_lossless(mk_f8e4m3b8nnz(false, -8, 0), mk_f16(false, -15, 0));
    check_convert_lossless(mk_f8e4m3b8nnz(true, -8, 0), mk_f16(true, 16, 1 << 9));

    for s in [false, true] {
        for m in 1u8..=0b111 {
            let shift = m.leading_zeros() - 4;
            check_convert_lossless(
                mk_f8e4m3b8nnz(s, -8, m),
                mk_f16(s, -7 - shift as i8, u16::from((m << shift) & 0b111) << 7),
            );
        }

        for e in -7..=7 {
            for m in 0..=0b111 {
                check_convert_lossless(mk_f8e4m3b8nnz(s, e, m), mk_f16(s, e, u16::from(m) << 7));
            }
        }
    }
}

#[test]
fn test_convert_f8e4m3nao_to_f16() {
    for s in [false, true] {
        check_convert_lossless(mk_f8e4m3nao(s, -7, 0), mk_f16(s, -15, 0));

        for m in 1u8..=0b111 {
            let shift = m.leading_zeros() - 4;
            check_convert_lossless(
                mk_f8e4m3nao(s, -7, m),
                mk_f16(s, -6 - shift as i8, u16::from((m << shift) & 0b111) << 7),
            );
        }

        for e in -6..=8 {
            for m in 0..=0b111 {
                if e == 8 && m == 0b111 {
                    check_convert_lossless(mk_f8e4m3nao(s, e, m), mk_f16(s, 16, 1 << 9));
                } else {
                    check_convert_lossless(mk_f8e4m3nao(s, e, m), mk_f16(s, e, u16::from(m) << 7));
                }
            }
        }
    }
}
