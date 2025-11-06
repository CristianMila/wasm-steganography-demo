use wasm_steganography::Steganography;
use wasm_steganography::Guest;

const SECRET: &str = "foo";

#[test]
fn decode_secret_valid_bmp_returns_secret() {
    // the previously encoded image secret should be "foo"
    let bmp_bytes = std::fs::read("tests/data/test-encoded.bmp").expect("failed loading the previously encoded image");
    let secret = Steganography::decode_secret_from_bmp(bmp_bytes.to_vec());

    assert_eq!(secret, SECRET);
}

#[test]
fn encode_secret_valid_bmp_returns_bmp_bytes() {
    let bmp_bytes = std::fs::read("tests/data/test.bmp").expect("couldn't read the non encoded image");
    let encoded_bmp_bytes = Steganography::encode_secret_into_bmp("foo".to_owned(), bmp_bytes.to_vec());
    assert_ne!(&bmp_bytes, &encoded_bmp_bytes);

    let previosly_encoded_img = std::fs::read("tests/data/test-encoded.bmp").expect("couldn't load the encoded image");
    assert_eq!(&encoded_bmp_bytes, &previosly_encoded_img);
}

#[test]
#[should_panic]
fn decode_secret_bmp_without_secret_panics() {
    // it has no secret encoded
    let bmp_bytes = std::fs::read("tests/data/test.bmp").expect("failed loading the non encoded image");
    let _secret = Steganography::decode_secret_from_bmp(bmp_bytes.to_vec());
}

#[test]
fn decode_jpg_should_succeed() {
    let image = std::fs::read("tests/data/test.jpeg").unwrap();
    let res = Steganography::encode_secret_into_jpeg(
        SECRET.to_string(),
        image,
    );

    assert!(res.is_empty() == false);

    let secret = Steganography::decode_secret_from_jpeg(res);    
    assert_eq!(secret, SECRET.to_string());
}
