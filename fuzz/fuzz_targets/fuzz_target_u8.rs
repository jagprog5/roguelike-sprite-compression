#![no_main]

use libfuzzer_sys::fuzz_target;
use sprite_sheet_compress::sprite_sheet_impl;

sprite_sheet_impl!(Img8, u8, u8);

fuzz_target!(|data: &[u8]| {
    let img = match Img8::decode(data) {
        Ok(v) => v,
        Err(_) => return, // the random bytes do not form a valid image (e.g. no magic string)
    };

    let img_encoded = match img.encode() {
        Ok(v) => v,
        Err(_) => return, // the image is not able to be compressed (e.g. palette size insufficient)
    };

    let img_encoded_decoded = Img8::decode(&img_encoded).unwrap();

    assert_eq!(img, img_encoded_decoded); // lossless
});

