# Flattening Quadratic Bézier Curves

Here I've implemented the algorithm presented by Raph Levien [here](https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html) and used it to render a few glyphs from some TrueType fonts. Here are 3 glyphs:

![Glyph '@'](media/glyph_36_smart_subdivision_test.png)
![Glyph 'F'](media/glyph_42_smart_subdivision_test.png)
![Glyph 'W'](media/glyph_59_smart_subdivision_test.png)

I've also implemented the De Casteljau method and rendered the same glyphs with it (images named `glyph_<index>_recursive_subdivision`). The differences are practically invisible, but Levien's method requires less number of segments generated and is _much_ faster.
