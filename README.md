# sprite io

This describes a file format for a compressed sprite.

All multibyte numbers are in big endian.

DimType: u32
PaletteIdType: u16

## Preamble

It starts with the magic string JAG_TILE_COMPRESS.

## Palette

Next is a PaletteIdType indicating the size of the following array. 
Next is an array of RGBA8888 color elements.  
This gives an array of colours which is selectable by a zero based index.

## Image

The next n bytes is the DimType width for the image.
The next n bytes is the DimType height for the image.

The file ends with an width * height array of PaletteIdType indices into the palette.
