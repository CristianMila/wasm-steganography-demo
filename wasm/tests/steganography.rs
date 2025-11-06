use wasm_steganography::Steganography;
use wasm_steganography::Guest;
use zune_image::codecs::qoi::zune_core;
use std::io::Cursor;
use zune_image::traits::StegoEncoder;

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
    let image = zune_image::image::Image::open("tests/data/test.jpeg").unwrap();
    let (width, height) = image.dimensions();
    let options = zune_core::options::EncoderOptions::new(width / 2, height / 2, zune_core::colorspace::ColorSpace::RGB, zune_core::bit_depth::BitDepth::Eight);
    options.set_quality(95);
    let mut zune_jpeg_encoder = zune_image::codecs::jpeg::JpegEncoder::new_with_options(options);
    let res = zune_jpeg_encoder.encode_with_secret(
        &image,
        SECRET.as_ref(),
    ).expect("failed to encode image as jpg with zune");

    assert!(res.is_empty() == false);

    std::fs::write("test-encoded-zune.jpg", &res).expect("failed to write the encoded image to disk");

    let decoded_image: Cursor<Vec<u8>> = Cursor::new(res);
    let decoder = image::codecs::jpeg::JpegDecoder::new(decoded_image).expect("error decoding the previously encoded steg image");
    let Some(secret) = decoder.get_secret() else {
        panic!("no secret was found")
    };
    
    assert_eq!(secret, SECRET.to_string());
}
