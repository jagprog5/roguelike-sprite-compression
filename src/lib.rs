#![feature(generic_const_exprs)] 

use num_traits::{PrimInt, Unsigned, ToBytes, FromBytes};
use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

// SIZE is a workaround for generic_const_exprs (incomplete rust feature); do not set
trait UInt<T=Self>: Unsigned + PrimInt + ToBytes + FromBytes {
    const SIZE_OK: (); // compile time restriction on UInt's size: cannot exceed usize

    fn to_be_bytes(&self) -> [u8; std::mem::size_of::<T>()];
}

impl<T> UInt for T
where
    T: Unsigned + PrimInt + ToBytes + FromBytes,
{
    const SIZE_OK: () = assert!(std::mem::size_of::<T>() <= std::mem::size_of::<usize>());

    fn to_be_bytes(&self) -> [u8; SIZE] {

    }
}

pub struct Image<DimType, PaletteIdType>
where
    DimType: UInt,
    PaletteIdType: UInt,
{
    width: DimType,
    pixels: Vec<Pixel>,                           // row major
    palette_id_dummy: PhantomData<PaletteIdType>, // generic param used only in impl
}

impl<DimType, PaletteIdType> Image<DimType, PaletteIdType>
where
    DimType: UInt,
    PaletteIdType: UInt,
{
    pub fn magic_string() -> &'static [u8] {
        b"JAG_TILE_COMPRESS"
    }

    pub fn height(&self) -> DimType {
        // panic occurs from expect only if pixels is manually set to something that is too long
        DimType::from(self.pixels.len() / (self.width.to_usize().unwrap())).expect("image height dim exceeded")
    }

    pub fn encode(&self) -> Result<Vec<u8>, &'static str> {
        let val: u32 = 0;
        let temp = val.to_be_bytes();

        // next available palette id
        let mut counter: PaletteIdType = PaletteIdType::zero();

        // associates each new color with its palette id
        let mut pixel_palette_map: HashMap<Pixel, PaletteIdType> = HashMap::new();

        // pixels converted to palette ids
        let mut paletted_data: Vec<PaletteIdType> = Vec::with_capacity(self.pixels.len());

        for &pixel in self.pixels.iter() {
            match pixel_palette_map.entry(pixel) {
                std::collections::hash_map::Entry::Occupied(v) => {
                    // if the pixel is present in the map, then use the existing palette id
                    paletted_data.push(*v.get());
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    // if the pixel isn't present in the map, then insert the palette id
                    if counter == PaletteIdType::max_value() {
                        return Err("colour palette exceeded");
                    }
                    let inserted = v.insert(counter);
                    paletted_data.push(*inserted);
                    counter = counter + PaletteIdType::one();
                }
            }
        }

        let mut palette_pixels: Vec<(PaletteIdType, Pixel)> = pixel_palette_map
            .drain()
            .map(|(pixel, id)| (id, pixel))
            .collect();

        palette_pixels.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));

        let mut ret: Vec<u8> = Vec::new();
        ret.extend_from_slice(Self::magic_string());

        // cast is guarded by counter check above
        let palette_pixels_len: PaletteIdType = PaletteIdType::from(palette_pixels.len()).unwrap();

        for b in palette_pixels_len.to_be_bytes() {
            ret.push(b);
        }

        for elem in palette_pixels {
            ret.push(elem.1.r);
            ret.push(elem.1.g);
            ret.push(elem.1.b);
            ret.push(elem.1.a);
        }

        for b in self.width.to_be_bytes().into() {
            ret.push(b);
        }

        for b in self.height().to_be_bytes().into() {
            ret.push(b);
        }

        for elem in paletted_data {
            for b in elem.to_be_bytes().into() {
                ret.push(b);
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

        let palette_size_bytes: [u8; PaletteIdType::SIZE] = data[0..PaletteIdType::SIZE]
            .try_into()
            .map_err(|_| "incomplete palette size")?;
        data = &data[2..];
        let palette_size = PaletteIdType::from_be_bytes(palette_size_bytes);

        let mut palette: Vec<Pixel> = Vec::with_capacity(palette_size.to_usize().unwrap());

        for _ in 0..palette_size {
            let pixel_bytes: [u8; 4] = data[0..4]
                .try_into()
                .map_err(|_| "incomplete palette pixel")?;
            data = &data[4..];
            palette.push(Pixel::from_bytes(pixel_bytes));
        }

        let width_bytes: [u8; 4] = data[0..4].try_into().map_err(|_| "incomplete width")?;
        data = &data[4..];
        let width = u32::from_be_bytes(width_bytes);

        let height_bytes: [u8; 4] = data[0..4].try_into().map_err(|_| "incomplete height")?;
        data = &data[4..];
        let height = u32::from_be_bytes(height_bytes);

        let image_size = width * height;
        let mut pixels: Vec<Pixel> = Vec::with_capacity(image_size.try_into().unwrap());

        for _ in 0..image_size {
            let pixel_bytes: [u8; 4] = data[0..4]
                .try_into()
                .map_err(|_| "incomplete palette pixel")?;
            data = &data[4..];
            pixels.push(Pixel::from_bytes(pixel_bytes));
        }

        Ok(Self { width, pixels })
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_encode_decode() {
        let img = Image {
            width: 2,
            pixels: vec![Pixel::red(), Pixel::green(), Pixel::red(), Pixel::blue()],
        };

        let encoded_img = img.encode().unwrap();

        let mut comparison_encoding: Vec<u8> = Vec::new();
        for ch in Self::magic_string() {
            comparison_encoding.push(*ch);
        }
        comparison_encoding.extend_from_slice(
            b"\x03\
            \xFF\x00\x00\xFF\
            \x00\xFF\x00\xFF",
        );
        // comparison_encoding.push(b"test");

        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
