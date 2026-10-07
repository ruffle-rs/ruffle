#!/usr/bin/env python3
"""Generates the SWFs that test.as loads, as Rascal cannot define shapes.

Each one holds a 2x2 shape filled with a 2x2 bitmap that is not smoothed:
- filled.swf places it on its only frame,
- nested.swf places it inside of a clip called "child",
- frames.swf places it on its second frame, where it stops.
solid.swf holds a 2x2 shape with a solid fill instead.
"""

import struct
import zlib

SIZE = 40  # twips, 2x2 pixels


class Bits:
    def __init__(self):
        self.bits = []

    def ub(self, n, v):
        self.bits += [(v >> i) & 1 for i in range(n - 1, -1, -1)]

    def sb(self, n, v):
        self.ub(n, v & ((1 << n) - 1))

    def bytes(self):
        b = self.bits + [0] * (-len(self.bits) % 8)
        return bytes(int("".join(map(str, b[i:i + 8])), 2) for i in range(0, len(b), 8))


def rect(x_min, x_max, y_min, y_max):
    b = Bits()
    n = 1 + max(abs(v) for v in (x_min, x_max, y_min, y_max)).bit_length()
    b.ub(5, n)
    for v in (x_min, x_max, y_min, y_max):
        b.sb(n, v)
    return b.bytes()


def tag(code, data, long=False):
    # Flash Player only accepts bitmap tags with the long header.
    if len(data) < 63 and not long:
        return struct.pack("<H", code << 6 | len(data)) + data
    return struct.pack("<HI", code << 6 | 63, len(data)) + data


def square_records():
    # One fill bit, no line bits; select fill 1, then draw the square.
    b = Bits()
    b.ub(4, 1)
    b.ub(4, 0)
    b.ub(1, 0)
    b.ub(5, 0b00101)
    b.ub(5, 1)
    b.sb(1, 0)
    b.sb(1, 0)
    b.ub(1, 1)
    n = SIZE.bit_length() + 1
    for dx, dy in ((SIZE, 0), (0, SIZE), (-SIZE, 0), (0, -SIZE)):
        b.ub(2, 0b11)
        b.ub(4, n - 2)
        b.ub(1, 0)
        if dx:
            b.ub(1, 0)
            b.sb(n, dx)
        else:
            b.ub(1, 1)
            b.sb(n, dy)
    b.ub(6, 0)
    return b.bytes()


def shape(character_id, fill_style):
    return tag(2, struct.pack("<H", character_id) + rect(0, SIZE, 0, SIZE) + bytes([1]) + fill_style
               + bytes([0]) + square_records())


def place(character_id, depth, name=None):
    if name is None:
        return tag(26, struct.pack("<BHH", 0x02, depth, character_id))
    return tag(26, struct.pack("<BHH", 0x22, depth, character_id) + name.encode() + b"\0")


def write(filename, frames, tags):
    body = rect(0, SIZE, 0, SIZE) + struct.pack("<HH", 12 << 8, frames) + b"".join(tags) + END
    with open(filename, "wb") as f:
        f.write(b"FWS" + bytes([8]) + struct.pack("<I", 8 + len(body)) + body)


SHOW_FRAME = tag(1, b"")
END = tag(0, b"")
STOP = tag(12, bytes([0x07, 0x00]))

# DefineBitsLossless, 32-bit ARGB: red, green / blue, yellow.
pixels = bytes([0, 255, 0, 0, 0, 0, 255, 0, 0, 0, 0, 255, 0, 255, 255, 0])
bitmap = tag(20, struct.pack("<HBHH", 1, 5, 2, 2) + zlib.compress(pixels), long=True)

# A clipped, non-smoothed bitmap fill (0x43) of bitmap 1, scaled by 20 (twips per pixel).
matrix = Bits()
matrix.ub(1, 1)
matrix.ub(5, 22)
matrix.sb(22, 20 << 16)
matrix.sb(22, 20 << 16)
matrix.ub(1, 0)
matrix.ub(5, 0)
filled_shape = shape(2, bytes([0x43]) + struct.pack("<H", 1) + matrix.bytes())

write("filled.swf", 1, [bitmap, filled_shape, place(2, 1), SHOW_FRAME])

sprite = tag(39, struct.pack("<HH", 3, 1) + place(2, 1) + SHOW_FRAME + END)
write("nested.swf", 1, [bitmap, filled_shape, sprite, place(3, 1, "child"), SHOW_FRAME])

write("frames.swf", 2, [bitmap, filled_shape, SHOW_FRAME, place(2, 1), STOP, SHOW_FRAME])

# A solid gray fill (0x00).
write("solid.swf", 1, [shape(1, bytes([0x00, 0x80, 0x80, 0x80])), place(1, 1), SHOW_FRAME])
