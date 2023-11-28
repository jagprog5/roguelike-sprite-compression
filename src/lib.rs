use static_assertions::const_assert;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
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
}

macro_rules! sprite {
    ($Name: ident, $DimType: ty, $PaletteIdType: ty) => {
        const_assert!(std::mem::size_of::<$DimType>() <= std::mem::size_of::<usize>());
        const_assert!(std::mem::size_of::<$PaletteIdType>() <= std::mem::size_of::<usize>());

        #[derive(PartialEq, Eq, Debug)]
        pub struct $Name {
            width: $DimType,
            pixels: Vec<Pixel>, // row major
        }

        impl $Name {
            pub fn magic_string() -> &'static [u8] {
                b"\xCAC1C7"
            }

            pub fn height(&self) -> $DimType {
                // panic occurs from expect only if pixels is manually set to something that is too long
                (self.pixels.len() / self.width as usize)
                    .try_into()
                    .expect("image height dim exceeded")
            }

            pub fn encode(&self) -> Result<Vec<u8>, &'static str> {
                // next available palette id
                let mut counter: $PaletteIdType = 0;

                // associates each new color with its palette id
                let mut pixel_palette_map: HashMap<Pixel, $PaletteIdType> = HashMap::new();

                // pixels converted to palette ids
                let mut paletted_data: Vec<$PaletteIdType> = Vec::with_capacity(self.pixels.len());

                for &pixel in self.pixels.iter() {
                    match pixel_palette_map.entry(pixel) {
                        std::collections::hash_map::Entry::Occupied(v) => {
                            // if the pixel is present in the map, then use the existing palette id
                            paletted_data.push(*v.get());
                        }
                        std::collections::hash_map::Entry::Vacant(v) => {
                            // if the pixel isn't present in the map, then insert the palette id
                            if counter == <$PaletteIdType>::MAX {
                                return Err("colour palette exceeded");
                            }
                            let inserted = v.insert(counter);
                            paletted_data.push(*inserted);
                            counter += 1;
                        }
                    }
                }

                let mut palette_pixels: Vec<($PaletteIdType, Pixel)> = pixel_palette_map
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

                for elem in paletted_data {
                    for b in elem.to_be_bytes().as_ref() {
                        ret.push(*b);
                    }
                }

                Ok(ret)
            }

            pub fn decode(data_arg: &[u8]) -> Result<Self, &'static str> {
                let mut data = &data_arg[..];

                if !data.starts_with(Self::magic_string()) {
                    return Err("magic string not found");
                }
                data = &data[Self::magic_string().len()..];

                let palette_size_bytes_ref = data.get(0..std::mem::size_of::<$PaletteIdType>()).ok_or("incomplete palette size")?;
                let palette_size_bytes: [u8; std::mem::size_of::<$PaletteIdType>()] = palette_size_bytes_ref.try_into().unwrap();
                let palette_size = <$PaletteIdType>::from_be_bytes(palette_size_bytes);
                data = &data[std::mem::size_of::<$PaletteIdType>()..];

                let mut palette: Vec<Pixel> = Vec::with_capacity(palette_size as usize);

                for _ in 0..palette_size {
                    let pixel_bytes_ref = data.get(0..4).ok_or("incomplete pixel")?;
                    let pixel_bytes = pixel_bytes_ref.try_into().unwrap();
                    palette.push(Pixel::from_bytes(pixel_bytes));
                    data = &data[4..];
                }

                let width_bytes_ref = data.get(0..std::mem::size_of::<$DimType>()).ok_or("incomplete width")?;
                let width_bytes: [u8; std::mem::size_of::<$DimType>()] = width_bytes_ref.try_into().unwrap();
                let width = <$DimType>::from_be_bytes(width_bytes);
                data = &data[std::mem::size_of::<$DimType>()..];

                let height_bytes_ref = data.get(0..std::mem::size_of::<$DimType>()).ok_or("incomplete height")?;
                let height_bytes: [u8; std::mem::size_of::<$DimType>()] = height_bytes_ref.try_into().unwrap();
                let height = <$DimType>::from_be_bytes(height_bytes);
                data = &data[std::mem::size_of::<$DimType>()..];

                let image_size = (width as usize).checked_mul(height as usize).ok_or("img dim overflow")?;
                let mut pixels: Vec<Pixel> = Vec::with_capacity(image_size);

                for _ in 0..image_size {
                    let palette_id_bytes_ref = data.get(0..std::mem::size_of::<$PaletteIdType>()).ok_or("incomplete palette id")?;
                    let palette_id_bytes: [u8; std::mem::size_of::<$PaletteIdType>()] = palette_id_bytes_ref.try_into().unwrap();
                    let palette_id = <$PaletteIdType>::from_be_bytes(palette_id_bytes);
                    let pixel = palette.get(palette_id as usize).ok_or("invalid palette id")?;
                    pixels.push(*pixel);
                    data = &data[std::mem::size_of::<$PaletteIdType>()..];   
                }

                Ok(Self { width, pixels })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        sprite!(Img, u32, u16);

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
            \x00\x02"
        );

        assert_eq!(img_encoding, correct_encoding);
        let image_decoded = Img::decode(&img_encoding).unwrap();
        assert_eq!(image_decoded, img);

        // a failure, incomplete palette id
        let correct_encoding_truncated = &correct_encoding[0..correct_encoding.len() - 1];
        Img::decode(&correct_encoding_truncated).unwrap_err();
    }
}
