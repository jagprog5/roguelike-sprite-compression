#![no_main]

use libfuzzer_sys::fuzz_target;
use sprite_sheet_compress::sprite_sheet_impl;

sprite_sheet_impl!(Img16, u16, u16);

fuzz_target!(|data: &[u8]| {
    let img = match Img16::decode(data) {
        Ok(v) => v,
        Err(_) => return, // the random bytes do not form a valid image (e.g. no magic string)
    };

    let img_encoded = match img.encode() {
        Ok(v) => v,
        Err(_) => return, // the image is not able to be compressed (e.g. palette size insufficient)
    };

    let img_encoded_decoded = Img16::decode(&img_encoded).unwrap();

    assert_eq!(img, img_encoded_decoded); // lossless
});
