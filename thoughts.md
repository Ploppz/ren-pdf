
#
```
graphics
graphics/font_ren
```
graphics actually draws to screen. It should know the PDF repr that we are trying to render.
It uses `font_ren` ETC

Additionally maybe OpenBook struct that has PdfReader + position + selections.. what not




# Font Renderer, Cache
- Read font file.
- Create cache

- `Font::layout()` -> iterator over `PositionedGlyph`
  - for each glyph here, 
- queue relevant glyphs to be cached & cache the queue
  - Cache will rasterize and return an u8 slice + Rect for uploading to GPU
- Make & upload geometry, find UV-coords by `Cache::rect_for`
