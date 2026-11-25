use d11_quality_lab::*;

#[test]
fn step1_encoder_exact_bytes_and_limit() {
    assert_eq!(encode(&[]), Ok(vec![0xa5, 0, 0]));
    assert_eq!(encode(&[7, 9]), Ok(vec![0xa5, 2, 7, 9, 12]));
    assert_eq!(encode(&[255]), Ok(vec![0xa5, 1, 255, 254]));
    assert_eq!(encode(&[0; 64]).unwrap().len(), 67);
    assert_eq!(encode(&[0; 65]), Err(FrameError::TooLong));
}

#[test]
fn step2_decoder_rejects_malformed_and_borrows() {
    let frame = [0xa5, 2, 7, 9, 12];
    let payload = decode(&frame).unwrap();
    assert_eq!(payload, &[7, 9]);
    assert!(std::ptr::eq(payload, &frame[2..4]));
    assert_eq!(decode(&[0xa5, 0, 0]), Ok(&[][..]));
    assert_eq!(decode(&[0xa5, 2]), Err(FrameError::TooShort));
    assert_eq!(decode(&[0xa4, 0, 0]), Err(FrameError::BadMagic));
    assert_eq!(decode(&[0xa5, 65, 0]), Err(FrameError::TooLong));
    assert_eq!(decode(&[0xa5, 2, 7, 12]), Err(FrameError::Length));
    assert_eq!(decode(&[0xa5, 0, 0, 0]), Err(FrameError::Length));
    assert_eq!(decode(&[0xa5, 2, 7, 9, 13]), Err(FrameError::Checksum));
}

#[test]
fn step3_loop_counts_inclusive_threshold() {
    assert_eq!(count_loop(&[0, 7, 9, 9, 255], 9), 3);
    assert_eq!(count_loop(&[0, 255], 0), 2);
    assert_eq!(count_loop(&[], 255), 0);
}

#[test]
fn step4_iterator_counts_inclusive_threshold() {
    assert_eq!(count_iter(&[0, 7, 9, 9, 255], 9), 3);
    assert_eq!(count_iter(&[0, 255], 255), 1);
    assert_eq!(count_iter(&[], 0), 0);
}
