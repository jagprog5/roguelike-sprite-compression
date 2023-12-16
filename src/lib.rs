pub use static_assertions::const_assert;
pub use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub fn from_bytes(data: [u8; 4]) -> Self {
        Pixel {
            r: data[0],
            g: data[1],
            b: data[2],
            a: data[3],
        }
    }

    pub fn red() -> Self {
        Pixel {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn green() -> Self {
        Pixel {
            r: 0,
            g: 255,
            b: 0,
            a: 255,
        }
    }

    pub fn blue() -> Self {
        Pixel {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        }
    }

    pub fn transparent_black() -> Self {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

#[macro_export]
macro_rules! sprite_sheet_impl {
    ($Name: ident, $DimType: ty, $PaletteIdType: ty) => {
        $crate::const_assert!(std::mem::size_of::<$DimType>() <= std::mem::size_of::<usize>());
        $crate::const_assert!(std::mem::size_of::<$PaletteIdType>() <= std::mem::size_of::<usize>());
        $crate::const_assert!(<$DimType>::MIN == 0);
        $crate::const_assert!(<$PaletteIdType>::MIN == 0);

        #[derive(PartialEq, Eq, Debug)]
        pub struct $Name {
            width: $DimType,
            pixels: Vec<$crate::Pixel>, // row major
        }

        impl $Name {
            pub fn magic_string() -> &'static [u8] {
                b"\xCAC1C7"
            }

            pub fn height(&self) -> $DimType {
                // panic occurs from expect only if pixels is manually set to something that is too long
                if self.width == 0 {
                    return 0;
                }
                (self.pixels.len() / self.width as usize)
                    .try_into()
                    .expect("image height dim exceeded. consider adjusting DimType")
            }

            pub fn encode(&self) -> Result<Vec<u8>, &'static str> {
                // next available palette id
                let mut counter: $PaletteIdType = 0;

                // associates each new color with its palette id
                let mut pixel_palette_map: $crate::HashMap<$crate::Pixel, $PaletteIdType> = $crate::HashMap::new();

                // pixels converted to palette ids, bytes
                let mut paletted_data: Vec<u8> =
                    Vec::with_capacity(self.pixels.len() * std::mem::size_of::<$PaletteIdType>());

                // run length so far
                let mut transparent_black_count: u8 = 0;

                // logic that occurs at the end of a transparent black background run
                let handle_bkg_run_end = |count: &mut u8, v: &mut Vec<u8>| {
                    if *count != 0 {
                        // paletted_data
                        for b in <$PaletteIdType>::MAX.to_be_bytes() {
                            v.push(b);
                        }
                        for b in count.to_be_bytes() {
                            v.push(b);
                        }
                        *count = 0;
                    }
                };

                for &pixel in self.pixels.iter() {
                    if pixel == $crate::Pixel::transparent_black() {
                        if transparent_black_count == u8::MAX {
                            // guard increment overflow (sets count to 0)
                            handle_bkg_run_end(&mut transparent_black_count, &mut paletted_data);
                        }
                        transparent_black_count += 1;
                        continue;
                    }

                    handle_bkg_run_end(&mut transparent_black_count, &mut paletted_data);

                    match pixel_palette_map.entry(pixel) {
                        std::collections::hash_map::Entry::Occupied(v) => {
                            let id = *v.get();
                            for b in id.to_be_bytes() {
                                paletted_data.push(b);
                            }
                        }
                        std::collections::hash_map::Entry::Vacant(v) => {
                            if counter == <$PaletteIdType>::MAX {
                                // MAX is reserved for background run length
                                return Err("colour palette length exceeded. consider adjusting PaletteIdType");
                            }
                            let id = *v.insert(counter);
                            for b in id.to_be_bytes() {
                                paletted_data.push(b);
                            }
                            counter += 1;
                        }
                    }
                }

                handle_bkg_run_end(&mut transparent_black_count, &mut paletted_data);

                let mut palette_pixels: Vec<($PaletteIdType, $crate::Pixel)> = pixel_palette_map
                    .drain()
                    .map(|(pixel, id)| (id, pixel))
                    .collect();

                palette_pixels.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));

                let mut ret: Vec<u8> = Vec::new();
                ret.extend_from_slice(Self::magic_string());

                // cast is guarded by pelette exceeded check above
                let palette_pixels_len = palette_pixels.len() as $PaletteIdType;

                for b in palette_pixels_len.to_be_bytes().as_ref() {
                    ret.push(*b);
                }

                for elem in palette_pixels {
                    ret.push(elem.1.r);
                    ret.push(elem.1.g);
                    ret.push(elem.1.b);
                    ret.push(elem.1.a);
                }

                for b in self.width.to_be_bytes().as_ref() {
                    ret.push(*b);
                }

                for b in self.height().to_be_bytes().as_ref() {
                    ret.push(*b);
                }

                ret.extend_from_slice(&paletted_data);
                Ok(ret)
            }

            pub fn decode(data_arg: &[u8]) -> Result<Self, &'static str> {
                let mut data = &data_arg[..];

                if !data.starts_with(Self::magic_string()) {
                    return Err("magic string not found");
                }
                data = &data[Self::magic_string().len()..];

                let palette_size_bytes_ref = data
                    .get(0..std::mem::size_of::<$PaletteIdType>())
                    .ok_or("incomplete palette size")?;
                let palette_size_bytes: [u8; std::mem::size_of::<$PaletteIdType>()] =
                    palette_size_bytes_ref.try_into().unwrap();
                let palette_size = <$PaletteIdType>::from_be_bytes(palette_size_bytes);
                data = &data[std::mem::size_of::<$PaletteIdType>()..];

                let mut palette: Vec<$crate::Pixel> = Vec::new();

                for _ in 0..palette_size {
                    let pixel_bytes_ref = data.get(0..4).ok_or("incomplete pixel")?;
                    let pixel_bytes = pixel_bytes_ref.try_into().unwrap();
                    palette.push($crate::Pixel::from_bytes(pixel_bytes));
                    data = &data[4..];
                }

                let width_bytes_ref = data
                    .get(0..std::mem::size_of::<$DimType>())
                    .ok_or("incomplete width")?;
                let width_bytes: [u8; std::mem::size_of::<$DimType>()] =
                    width_bytes_ref.try_into().unwrap();
                let width = <$DimType>::from_be_bytes(width_bytes);
                data = &data[std::mem::size_of::<$DimType>()..];

                let height_bytes_ref = data
                    .get(0..std::mem::size_of::<$DimType>())
                    .ok_or("incomplete height")?;
                let height_bytes: [u8; std::mem::size_of::<$DimType>()] =
                    height_bytes_ref.try_into().unwrap();
                let height = <$DimType>::from_be_bytes(height_bytes);
                data = &data[std::mem::size_of::<$DimType>()..];

                let image_size = (width as usize)
                    .checked_mul(height as usize)
                    .ok_or("img dim overflow")?;

                let mut pixels: Vec<$crate::Pixel> = Vec::new();

                while pixels.len() < image_size {
                    let palette_id_bytes_ref = data
                        .get(0..std::mem::size_of::<$PaletteIdType>())
                        .ok_or("incomplete palette id")?;
                    let palette_id_bytes: [u8; std::mem::size_of::<$PaletteIdType>()] =
                        palette_id_bytes_ref.try_into().unwrap();
                    let palette_id = <$PaletteIdType>::from_be_bytes(palette_id_bytes);
                    data = &data[std::mem::size_of::<$PaletteIdType>()..];

                    if (palette_id == <$PaletteIdType>::MAX) {
                        let run_count_ref = data.get(0..1).ok_or("incomplete run size")?;
                        let run_count_bytes: [u8; 1] = run_count_ref.try_into().unwrap();
                        data = &data[1..];
                        let run_count = run_count_bytes[0];
                        if pixels.len() + run_count as usize > image_size {
                            return Err("overlong run length at end");
                        }
                        for _ in 0..run_count {
                            pixels.push($crate::Pixel::transparent_black());
                        }
                    } else {
                        let pixel = palette
                            .get(palette_id as usize)
                            .ok_or("invalid palette id")?;
                        pixels.push(*pixel);
                    }
                }

                Ok(Self { width, pixels })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    sprite_sheet_impl!(Img, u32, u16);

    #[test]
    fn encode_decode() {
        let img = Img {
            width: 2u32,
            pixels: vec![Pixel::red(), Pixel::green(), Pixel::red(), Pixel::blue()],
        };

        let img_encoding = img.encode().unwrap();

        let mut correct_encoding: Vec<u8> = Vec::new();

        for ch in Img::magic_string() {
            correct_encoding.push(*ch);
        }
        correct_encoding.extend_from_slice(
            b"\x00\x03\
            \xFF\x00\x00\xFF\
            \x00\xFF\x00\xFF\
            \x00\x00\xFF\xFF\
            \x00\x00\x00\x02\
            \x00\x00\x00\x02\
            \x00\x00\
            \x00\x01\
            \x00\x00\
            \x00\x02",
        );

        assert_eq!(img_encoding, correct_encoding);
        let image_decoded = Img::decode(&img_encoding).unwrap();
        assert_eq!(image_decoded, img);

        // a failure, incomplete palette id
        let correct_encoding_truncated = &correct_encoding[0..correct_encoding.len() - 1];
        Img::decode(&correct_encoding_truncated).unwrap_err();
    }

    #[test]
    fn run_length_check() {
        let img = Img {
            width: 2u32,
            pixels: vec![
                Pixel::red(),
                Pixel::transparent_black(),
                Pixel::transparent_black(),
                Pixel::red(),
            ],
        };

        let img_encoding = img.encode().unwrap();

        let mut correct_encoding: Vec<u8> = Vec::new();

        for ch in Img::magic_string() {
            correct_encoding.push(*ch);
        }
        correct_encoding.extend_from_slice(
            b"\x00\x01\
            \xFF\x00\x00\xFF\
            \x00\x00\x00\x02\
            \x00\x00\x00\x02\
            \x00\x00\
            \xFF\xFF\x02\
            \x00\x00",
        );

        assert_eq!(img_encoding, correct_encoding);
        let image_decoded = Img::decode(&img_encoding).unwrap();
        assert_eq!(image_decoded, img);
    }

    #[test]
    fn long_run_length() {
        let mut v: Vec<Pixel> = Vec::new();
        v.push(Pixel::red());
        for _ in 0..23 * 23 - 2 {
            // total size 23x23
            v.push(Pixel::transparent_black());
        }
        v.push(Pixel::red());

        let img = Img {
            width: 23u32,
            pixels: v,
        };

        let img_encoding = img.encode().unwrap();

        let mut correct_encoding: Vec<u8> = Vec::new();

        for ch in Img::magic_string() {
            correct_encoding.push(*ch);
        }
        correct_encoding.extend_from_slice(
            b"\x00\x01\
            \xFF\x00\x00\xFF\
            \x00\x00\x00\x17\
            \x00\x00\x00\x17\
            \x00\x00\
            \xFF\xFF\xFF\
            \xFF\xFF\xFF\
            \xFF\xFF\x11\
            \x00\x00",
        );

        assert_eq!(img_encoding, correct_encoding);
        let image_decoded = Img::decode(&img_encoding).unwrap();
        assert_eq!(image_decoded, img);
    }
}
