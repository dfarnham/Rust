use super::*;

#[test]
fn test_basic() {
    let mut src: [u8; 3] = [0; 3];
    let mut dst: [u8; 4] = [0; 4];

    src[0] = 'A' as u8;
    src[1] = 'B' as u8;
    src[2] = 'C' as u8;

    b64_encode(src, &mut dst, 3);

    assert!(dst[0] as char == 'Q');
    assert!(dst[1] as char == 'U');
    assert!(dst[2] as char == 'J');
    assert!(dst[3] as char == 'D');
}

#[test]
fn test_basic_decode() {
    let mut src: [u8; 3] = [0; 3];
    let mut dst: [u8; 4] = [0; 4];

    dst[0] = 'Q' as u8;
    dst[1] = 'U' as u8;
    dst[2] = 'J' as u8;
    dst[3] = 'D' as u8;

    let nbytes = b64_decode(dst, &mut src);

    assert!(nbytes == 3);
    assert!(src[0] as char == 'A');
    assert!(src[1] as char == 'B');
    assert!(src[2] as char == 'C');
}

#[test]
fn test_padded_encode1() {
    let mut src: [u8; 3] = [0; 3];
    let mut dst: [u8; 4] = [0; 4];

    src[0] = 'A' as u8;
    src[1] = '*' as u8; // invalid Base64 char
    src[2] = '*' as u8; // invalid Base64 char

    b64_encode(src, &mut dst, 1);

    assert!(dst[0] as char == 'Q');
    assert!(dst[1] as char == 'Q');
    assert!(dst[2] as char == '=');
    assert!(dst[3] as char == '=');
}

#[test]
fn test_padded_decode1() {
    let mut src: [u8; 3] = [0; 3];
    let mut dst: [u8; 4] = [0; 4];

    dst[0] = 'Q' as u8;
    dst[1] = 'Q' as u8;
    dst[2] = '=' as u8;
    dst[3] = '=' as u8;

    let nbytes = b64_decode(dst, &mut src);

    assert!(nbytes == 1);
    assert!(src[0] as char == 'A');
}

#[test]
fn test_padded_encode2() {
    let mut src: [u8; 3] = [0; 3];
    let mut dst: [u8; 4] = [0; 4];

    src[0] = 'A' as u8;
    src[1] = 'B' as u8;
    src[2] = '*' as u8; // invalid Base64 char

    b64_encode(src, &mut dst, 2);

    assert!(dst[0] as char == 'Q');
    assert!(dst[1] as char == 'U');
    assert!(dst[2] as char == 'I');
    assert!(dst[3] as char == '=');
}

#[test]
fn test_padded_decode2() {
    let mut src: [u8; 3] = [0; 3];
    let mut dst: [u8; 4] = [0; 4];

    dst[0] = 'Q' as u8;
    dst[1] = 'U' as u8;
    dst[2] = 'I' as u8;
    dst[3] = '=' as u8;

    let nbytes = b64_decode(dst, &mut src);

    assert!(nbytes == 2);
    assert!(src[0] as char == 'A');
    assert!(src[1] as char == 'B');
}
