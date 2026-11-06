#!/usr/bin/env python3
r"""Add two test fonts to a compiled AS3 SWF.

    python3 add_fonts.py IN.swf OUT.swf

IN.swf is Test.as compiled by any AS3 compiler (mxmlc, or asc as shown
below). OUT.swf is IN.swf with two DefineFont3 tags inserted before its
first DoABC tag, written uncompressed:

* "SmallTextFont" (id 1) has the SmallText flag set (font flags 0xa4). This is
  the flag the Flash authoring tool sets for "Bitmap text [no anti-alias]";
  mxmlc's [Embed] has no option for it, which is why this script exists.
* "PlainFont" (id 2) is the same font without the flag (font flags 0x84).

Every glyph is a solid bar. Advances are given in a 1024 unit em (stored at
20x, as DefineFont3 requires) and were picked so that they fall between
whole pixels at the sizes Test.as uses.

To rebuild test.swf with the asc.jar in tools/asc (run from this directory;
playerglobal_import.abc is written by ruffle_core's build script into
target/<profile>/build/ruffle_core-*/out/):

    java -cp ../../../../../tools/asc/asc.jar macromedia.asc.embedding.Main \
        -AS3 -md -optimize -import path/to/playerglobal_import.abc \
        -swf Test,400,300,24 Test.as
    python3 add_fonts.py Test.swf test.swf
    rm Test.swf Test.abc
"""
import struct
import sys
import zlib

# (character, advance in 1/1024 em)
GLYPHS = [
    (" ", 256),
    ("a", 440),
    ("b", 512),
    ("c", 257),
    ("d", 600),
    ("k", 440),
]
# (left, right, adjustment in 1/1024 em)
KERNING = [("k", "k", -150)]
ASCENT, DESCENT, LEADING = 900, 200, 0
BAR_WIDTH, BAR_HEIGHT = 154, 716
SCALE = 20  # DefineFont3 coordinates are 20 times those of DefineFont2.


class BitWriter:
    def __init__(self):
        self.out = bytearray()
        self.acc = 0
        self.count = 0

    def bits(self, value, n):
        for i in range(n - 1, -1, -1):
            self.acc = (self.acc << 1) | ((value >> i) & 1)
            self.count += 1
            if self.count == 8:
                self.out.append(self.acc)
                self.acc = 0
                self.count = 0

    def finish(self):
        if self.count:
            self.out.append((self.acc << (8 - self.count)) & 0xFF)
            self.acc = 0
            self.count = 0
        return bytes(self.out)


def signed_bits(*values):
    return max(max(v.bit_length() if v >= 0 else (~v).bit_length() for v in values) + 1, 1)


def rect(x_min, x_max, y_min, y_max):
    w = BitWriter()
    n = signed_bits(x_min, x_max, y_min, y_max)
    w.bits(n, 5)
    for v in (x_min, x_max, y_min, y_max):
        w.bits(v & ((1 << n) - 1), n)
    return w.finish()


def bar_shape(width, height):
    """A SHAPE record: one fill style bit, no line style bits, a filled box."""
    w = BitWriter()
    w.bits(1, 4)  # NumFillBits
    w.bits(0, 4)  # NumLineBits
    if width > 0:
        # StyleChangeRecord: StateMoveTo and StateFillStyle0, move to (0, 0).
        w.bits(0, 1)
        w.bits(0b00011, 5)
        w.bits(1, 5)
        w.bits(0, 1)
        w.bits(0, 1)
        w.bits(1, 1)  # FillStyle0 = 1
        for vertical, delta in ((0, width), (1, -height), (0, -width), (1, height)):
            n = max(signed_bits(delta), 2)
            w.bits(1, 1)  # edge
            w.bits(1, 1)  # straight
            w.bits(n - 2, 4)
            w.bits(0, 1)  # not a general line
            w.bits(vertical, 1)
            w.bits(delta & ((1 << n) - 1), n)
    w.bits(0, 6)  # EndShapeRecord
    return w.finish()


def tag(code, body):
    return struct.pack("<HI", (code << 6) | 0x3F, len(body)) + body


def define_font_3(font_id, name, flags):
    shapes = [bar_shape(0 if c == " " else BAR_WIDTH * SCALE, BAR_HEIGHT * SCALE) for c, _ in GLYPHS]
    count = len(GLYPHS)
    offsets = []
    offset = 2 * (count + 1)
    for shape in shapes:
        offsets.append(offset)
        offset += len(shape)
    assert offset <= 0xFFFF, "font too big for 16-bit offsets"
    encoded_name = name.encode("ascii")
    body = struct.pack("<HBBB", font_id, flags, 1, len(encoded_name)) + encoded_name
    body += struct.pack("<H", count)
    body += b"".join(struct.pack("<H", o) for o in offsets)
    body += struct.pack("<H", offset)  # CodeTableOffset
    body += b"".join(shapes)
    body += b"".join(struct.pack("<H", ord(c)) for c, _ in GLYPHS)
    body += struct.pack("<HHh", ASCENT * SCALE, DESCENT * SCALE, LEADING * SCALE)
    body += b"".join(struct.pack("<h", a * SCALE) for _, a in GLYPHS)
    body += b"".join(
        rect(0, 0 if c == " " else BAR_WIDTH * SCALE, -BAR_HEIGHT * SCALE, 0) for c, _ in GLYPHS
    )
    body += struct.pack("<H", len(KERNING))
    body += b"".join(struct.pack("<HHh", ord(l), ord(r), k * SCALE) for l, r, k in KERNING)
    return tag(75, body)


def read_swf(data):
    if data[:3] == b"CWS":
        data = b"FWS" + data[3:8] + zlib.decompress(data[8:])
    elif data[:3] != b"FWS":
        raise SystemExit("expected an FWS or CWS file")
    rect_bits = data[8] >> 3
    header_end = 8 + (5 + 4 * rect_bits + 7) // 8 + 4
    tags = []
    pos = header_end
    while pos < len(data):
        code_and_length = struct.unpack_from("<H", data, pos)[0]
        start = pos
        pos += 2
        length = code_and_length & 0x3F
        if length == 0x3F:
            length = struct.unpack_from("<I", data, pos)[0]
            pos += 4
        pos += length
        tags.append((code_and_length >> 6, data[start:pos]))
    return data[:header_end], tags


def main():
    src, dst = sys.argv[1:3]
    header, tags = read_swf(open(src, "rb").read())
    fonts = define_font_3(1, "SmallTextFont", 0xA4) + define_font_3(2, "PlainFont", 0x84)
    out = bytearray(header)
    inserted = False
    for code, raw in tags:
        if code in (75, 48, 10):
            raise SystemExit("IN.swf already has fonts")
        if code in (72, 82) and not inserted:  # DoABC, DoABC2
            out += fonts
            inserted = True
        out += raw
    if not inserted:
        raise SystemExit("no DoABC tag found")
    out[4:8] = struct.pack("<I", len(out))
    open(dst, "wb").write(bytes(out))


if __name__ == "__main__":
    main()
