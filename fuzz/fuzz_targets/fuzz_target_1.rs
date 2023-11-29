#![no_main]

use libfuzzer_sys::fuzz_target;
use sprite_sheet_compress::sprite_sheet_impl;

type DimType = u16;
type PaletteIdType = u16;
sprite_sheet_impl!(Img, DimType, PaletteIdType);

fuzz_target!(|data: &[u8]| {
    let img = match Img::decode(data) {
        Ok(v) => v,
        Err(_) => return, // the random bytes do not form a valid image (e.g. no magic string)
    };

    let img_encoded = match img.encode() {
        Ok(v) => v,
        Err(_) => return, // the image is not able to be compressed (e.g. palette size insufficient)
    };

    let img_encoded_decoded = Img::decode(&img_encoded).unwrap();

    assert_eq!(img, img_encoded_decoded); // lossless
});
