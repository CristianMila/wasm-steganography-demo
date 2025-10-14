mod bindings;

extern crate alloc;

use std::io::Cursor;
use image::{ImageReader, Rgb};
pub use bindings::Guest;

bindings::export!(Steganography with_types_in bindings);

pub struct Steganography;

impl Guest for Steganography {
    fn encode_secret_into_bmp(secret: String, image: Vec<u8>) -> Vec<u8> {
        let cloned_image = image.to_vec();
        let image = ImageReader::new(Cursor::new(&cloned_image)).with_guessed_format().unwrap().decode().unwrap();
        let image::DynamicImage::ImageRgb8(mut img_buf) = image else {
            panic!("image format not supported")
        };

        u64::checked_mul(secret.len() as u64, u8::BITS as u64).expect("the string is too long");

        let mut secret_len = secret.len() as u64; // shadowing to make it mutable but keep semantics
        let mut pixel_iter = img_buf.enumerate_pixels_mut();

        for _ in (0..u64::BITS+2).step_by(3) {
            secret_len <<= 3;

            let mut pixel = pixel_iter.next().expect("exhausted pixel iterator before encoding all secret characters");
            encode_bits_in_rgb_byte(&mut pixel.2, (secret_len >> 56) as u8);
        }

        for character in secret.as_bytes() {
            let mut pixel = pixel_iter.next().expect("exhausted pixel iterator before encoding all secret characters");
            encode_bits_in_rgb_byte(&mut pixel.2, *character);

            let mut pixel = pixel_iter.next().expect("exhausted pixel iterator before encoding all secret characters");
            encode_bits_in_rgb_byte(&mut pixel.2, *character << 3);

            let mut pixel = pixel_iter.next().expect("exhausted pixel iterator before encoding all secret characters");
            encode_bits_in_rgb_byte(&mut pixel.2, *character << 6);
        }

        let mut ret = Vec::<u8>::new();
        let mut writer = Cursor::new(&mut ret);
        img_buf.write_to(&mut writer, image::ImageFormat::Bmp).expect("failed to write the image to the writer");

        ret
    }

    fn decode_secret_from_bmp(image: Vec <u8>) -> String {
        let image = ImageReader::new(Cursor::new(&image)).with_guessed_format().unwrap().decode().unwrap();
        let image::DynamicImage::ImageRgb8(img_buf) = image else {
            panic!("Image format not supported. Only BMP with 24bits of depth at the moment.")
        };

        let mut pixel_iter = img_buf.enumerate_pixels();
        let mut secret_len = 0u64;

        for _ in (0..u64::BITS+2).step_by(3) {
            let pixel = pixel_iter.next().expect("exhausted pixel iterator before encoding all secret characters");
            secret_len = secret_len << 3 | decode_bits_from_rgb_byte(pixel.2);
        }

        secret_len >>= 5; // accounting for the 2 useless bits from the last pixel + 3 bits (???)

        let mut secret: Vec<u8> = Vec::new(); 

        for _ in 0..secret_len {
            let mut character: u8;
            let pixel = pixel_iter.next().expect("exhausted pixel iterator before encoding all secret characters");
            character = decode_bits_from_rgb_byte(pixel.2) as u8;

            let pixel = pixel_iter.next().expect("exhausted pixel iterator before encoding all secret characters");
            character = character << 3 | decode_bits_from_rgb_byte(pixel.2) as u8;

            let pixel = pixel_iter.next().expect("exhausted pixel iterator before encoding all secret characters");
            character = character << 2 | (decode_bits_from_rgb_byte(pixel.2) as u8 >> 1);
            //                       ^ this one has to account for the "lost" bit when encoding a
            //                       byte in 3 pixels (9 bits, last one always 0)

            secret.push(character);
        }

        let secret_str = str::from_utf8(&secret).expect("secret was not valid utf8");

        secret_str.to_string()
    }
}

fn encode_bits_in_rgb_byte(rgb_pixel: &mut Rgb<u8>, message_byte: u8) {
    // set original lsb to 0
    rgb_pixel.0[0] &= 0xFE;
    rgb_pixel.0[1] &= 0xFE;
    rgb_pixel.0[2] &= 0xFE;

    // get the new desired lsb
    let lsb_red     = message_byte >> 7;
    let lsb_green   = message_byte >> 6 & 0x1;
    let lsb_blue    = message_byte >> 5 & 0x1;

    // set it
    rgb_pixel.0[0] |= lsb_red;
    rgb_pixel.0[1] |= lsb_green;
    rgb_pixel.0[2] |= lsb_blue;
}

fn decode_bits_from_rgb_byte(rgb_byte: &Rgb<u8>) -> u64 {

    let bit_0 = rgb_byte.0[0] & 1;
    let bit_1 = rgb_byte.0[1] & 1;
    let bit_2 = rgb_byte.0[2] & 1;

    let mut decoded_bits = bit_0 as u64;
    decoded_bits <<= 1;
    decoded_bits |= bit_1 as u64;
    decoded_bits <<= 1;
    decoded_bits |= bit_2 as u64;

    decoded_bits
}
