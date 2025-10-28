use image::ImageDecoder;
use wasm_steganography::Steganography;
use wasm_steganography::Guest;
use image::codecs::jpeg::{JpegDecoder, JpegEncoder};
use std::io::Cursor;

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
fn encode_jpg_should_succeed() {
    // it has no secret encoded
    let bmp_bytes = std::fs::read("tests/data/test.jpeg").expect("failed loading the non encoded image");
    let image = image::ImageReader::new(std::io::Cursor::new(&bmp_bytes))
        .with_guessed_format()
        .expect("failed to guess image format")
        .decode()
        .expect("failed to decode image");
    let mut encoded_image: Vec<u8> = vec![];
    let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut encoded_image);
    encoder.encode_image_with_secret(&image, SECRET.to_string()).expect("failed to encode image as jpg");

    let decoded_image: Cursor<Vec<u8>> = Cursor::new(encoded_image);
    let decoder = image::codecs::jpeg::JpegDecoder::new(decoded_image).expect("error decoding the previously encoded steg image");
    let Some(secret) = decoder.get_secret() else {
        panic!("no secret was found")
    };
    
    assert_eq!(secret, SECRET.to_string());
}
