use swfp::FpStatus;

const FLAGS: [FpStatus; 5] = [
    FpStatus::INVALID,
    FpStatus::DIV_BY_ZERO,
    FpStatus::OVERFLOW,
    FpStatus::UNDERFLOW,
    FpStatus::INEXACT,
];

#[test]
fn test_flags() {
    assert!(FpStatus::OK.is_ok());
    assert!(FpStatus::OK.contains(FpStatus::OK));
    assert_eq!(FpStatus::OK | FpStatus::OK, FpStatus::OK);
    assert_eq!(FpStatus::OK & FpStatus::OK, FpStatus::OK);

    for (i, flag1) in FLAGS.into_iter().enumerate() {
        assert!(!flag1.is_ok());
        assert_ne!(flag1, FpStatus::OK);
        assert!(flag1.contains(FpStatus::OK));
        assert!(!FpStatus::OK.contains(flag1));
        assert_eq!(flag1 | FpStatus::OK, flag1);
        assert_eq!(flag1 & FpStatus::OK, FpStatus::OK);

        for (j, flag2) in FLAGS.into_iter().enumerate() {
            assert_eq!(flag1 == flag2, i == j);
            assert_eq!(flag1.contains(flag2), i == j);
            assert_eq!((flag1 & flag2).is_ok(), i != j);

            let both = flag1 | flag2;
            assert_eq!(both, flag2 | flag1);
            assert!(both.contains(flag1));
            assert!(both.contains(flag2));
            assert_eq!(both & flag1, flag1);
            assert_eq!(both & flag2, flag2);

            let mut status = flag1;
            status |= flag2;
            assert_eq!(status, both);
            status &= flag2;
            assert_eq!(status, flag2);
        }
    }
}

#[test]
fn test_fmt_debug() {
    assert_eq!(format!("{:?}", FpStatus::OK), "OK");
    assert_eq!(format!("{:?}", FpStatus::INVALID), "INVALID");
    assert_eq!(format!("{:?}", FpStatus::DIV_BY_ZERO), "DIV_BY_ZERO");
    assert_eq!(format!("{:?}", FpStatus::OVERFLOW), "OVERFLOW");
    assert_eq!(format!("{:?}", FpStatus::UNDERFLOW), "UNDERFLOW");
    assert_eq!(format!("{:?}", FpStatus::INEXACT), "INEXACT");
    assert_eq!(
        format!("{:?}", FpStatus::OVERFLOW | FpStatus::INEXACT),
        "OVERFLOW | INEXACT",
    );
    assert_eq!(
        format!("{:?}", FpStatus::INEXACT | FpStatus::UNDERFLOW),
        "UNDERFLOW | INEXACT",
    );

    let all = FLAGS.into_iter().fold(FpStatus::OK, |acc, flag| acc | flag);
    assert_eq!(
        format!("{all:?}"),
        "INVALID | DIV_BY_ZERO | OVERFLOW | UNDERFLOW | INEXACT",
    );
}
