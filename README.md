# sprite io

```bash
cargo test # run tests

cargo install cargo-fuzz # fuzzer dep
rustup install nightly
rustup default nightly

cargo fuzz run fuzz_target_u16 # run fuzzer
```

This is a basic lossless image compressions lib intended for spite sheets. It implements:

1. A palette lookup table.
2. Run length encoding, used only for transparent black pixels (0000).

Compile time parameters:  
`DimType`: the type which stores the image's dimensions.  
`PaletteIdType`: the type which stores indices within the palette lookup (error if too many colours used).
 
# File Format

All multibyte numbers are in big endian. The following elements are listed in the same order as in the file:

## Preamble

It starts with the magic string in hex: CA, C1, C7.

## Palette

A `PaletteIdType` indicating the size of the palette lookup array.  
This is followed by the array of RGBA8888 elements. It is zero indexed later.

## Dimensions

`DimType` width for image.  
`DimType` height for image.

## Pixels

The file ends with an array of `PaletteIdType` indices into the palette, representing each pixel value in the image (row major).

However, the value `PaletteIdType::MAX` is reserved for a special meaning. The following byte after it indicates the number of entirely transparent black pixels that follow. For example,

10 transparent black pixels in a row => MAX, 10

If more than 255 of these pixels are in a row, then this is encoded by multiple `<MAX, count>` pairs.
