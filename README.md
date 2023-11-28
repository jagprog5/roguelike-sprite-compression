# sprite io

This gives a basic image compression lib based on a palett lookup table.

(all multibyte numbers are in big endian).

Template parameters (tunable at compile time):
DimType: u32
PaletteIdType: u16

The following describes the file format:

## Preamble

It starts with the magic string bit pattern in hex: CA, C1, C7.

## Palette

Next is a PaletteIdType indicating the size of the following array. 
Next is an array of RGBA8888 color elements.  
This gives an array of colours which is selectable by a zero based index.

## Image

The next n bytes is the DimType width for the image.
The next n bytes is the DimType height for the image.

The file ends with an width * height array of PaletteIdType indices into the palette (row major).
