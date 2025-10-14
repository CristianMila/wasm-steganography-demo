use assert_cmd::Command;
use assert_fs::NamedTempFile;

#[test]
fn encode_valid_bmp_returns_image() {
    let output_file = NamedTempFile::new("output.bmp").unwrap();
    let mut cmd = Command::cargo_bin("cli").unwrap();
    let res = cmd.args(&[
        "encode",
        "-i",
        "tests/data/test.bmp",
        "-o",
        output_file.to_str().expect("failed to get the path of the temp output file"),
        "-s",
        "foo"
    ]).ok();

    assert!(res.is_ok());

    let previously_encoded_img = std::fs::read("tests/data/test-encoded.bmp").expect("failed reading previously encoded image");
    let encoded_img_bytes = std::fs::read(output_file).expect("failed reading encoded image");

    assert_eq!(&encoded_img_bytes, &previously_encoded_img);
}

#[test]
fn decode_valid_bmp_returns_secret() {
    let mut cmd = Command::cargo_bin("cli").unwrap();
    let res = cmd.args(&[
        "decode",
        "-i",
        "tests/data/test-encoded.bmp",
    ]).ok();

    assert!(res.is_ok());

    let output = res.expect("didn't get any output from the command");
    let stdout = std::str::from_utf8(&output.stdout).expect("could't parse the result into a utf8 string").trim_end(); 

    assert_eq!(stdout, "foo");
}

#[test]
fn decode_unencoded_bmp_panics() {
    let mut cmd = Command::cargo_bin("cli").unwrap();
    let res = cmd.args(&[
        "decode",
        "-i",
        "tests/data/test.bmp",
    ]).ok();

    assert!(res.is_err());
}
