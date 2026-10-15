#!/usr/bin/env python3
"""Writes test.swf for visual/static_text_offstage (standard library only).

Static text (DefineText) wholly outside the stage is not drawn. Flash Player
decides that from the bounds stored in the DefineText tag, not from the
glyphs: text whose bounds miss the stage is skipped even when its glyphs would
reach the stage, and text whose bounds touch the stage is drawn in full, every
line of it. (A text field is different: Flash Player leaves out each line after
the first that is not wholly inside the field.)

AVM1, SWF 8, 400x300, white, 3 frames. The font has one glyph, a solid square
filling the em, so at a text height of 40 px each glyph is a 40x40 square with
its edges on whole pixels and nothing is anti-aliased. Each text is two glyphs,
45 px apart. Two 4x4 magenta squares mark the stage corners (0,0) and
(396,296); they are how the Flash Player screenshot was cropped to the stage.
"""

import struct
from pathlib import Path

BLACK, RED, GREEN, BLUE, CYAN, MAGENTA = (
    (0, 0, 0), (255, 0, 0), (0, 255, 0), (0, 0, 255), (0, 255, 255), (255, 0, 255))
FONT = 1


def bits(*fields):
    """Packs (value, width) pairs, most significant bit first, zero-padded to a byte."""
    s = "".join(format(value & ((1 << width) - 1), f"0{width}b") for value, width in fields)
    s += "0" * (-len(s) % 8)
    return bytes(int(s[i:i + 8], 2) for i in range(0, len(s), 8))


def rect(x0, y0, x1, y1):
    """RECT, given in pixels and stored in twips (1/20 px)."""
    return bits((16, 5), *((round(v * 20), 16) for v in (x0, x1, y0, y1)))


def matrix(x=0, y=0):
    """MATRIX with only a translation, in pixels."""
    return bits((0, 1), (0, 1), (16, 5), (round(x * 20), 16), (round(y * 20), 16))


def tag(code, body=b""):
    return struct.pack("<HI", code << 6 | 0x3F, len(body)) + body


def edge(vertical, delta):
    """Straight edge record: a horizontal or vertical line, delta in em units."""
    return (1, 1), (1, 1), (14, 4), (0, 1), (vertical, 1), (delta, 16)


def define_font():
    """DefineFont with one glyph, the square (0,-1024)..(1024,0) in em units."""
    shape = bits(
        (1, 4), (0, 4),  # 1 bit per fill style index, no line styles
        (0, 1), (0b00011, 5), (16, 5), (0, 16), (0, 16), (1, 1),  # move to (0,0), FillStyle0 = 1
        *edge(0, 1024), *edge(1, -1024), *edge(0, -1024), *edge(1, 1024),
        (0, 6))  # end of shape
    return tag(10, struct.pack("<HH", FONT, 2) + shape)


def define_text(cid, x, y, rgb, bounds=None, size=40, glyphs=2, lines=1):
    """DefineText: squares of size px, 5 px apart, the first with its top left at (x, y).

    Each further line starts size + 10 px below the one before.
    bounds: the TextBounds to store, in pixels; by default the glyphs' real box.
    """
    pitch = size + 10
    bounds = bounds or (x, y, x + glyphs * (size + 5) - 5, y + (lines - 1) * pitch + size)
    body = struct.pack("<H", cid) + rect(*bounds) + matrix()
    body += bytes([1, 16])  # bits per glyph index and per advance
    for line in range(lines):
        baseline = (y + line * pitch + size) * 20
        if line == 0:
            # The first text record sets font, colour, x, y (the baseline) and height.
            body += bytes([0x8F]) + struct.pack("<H3BhhH", FONT, *rgb, x * 20, baseline, size * 20)
        else:
            # Each later one only moves to the start of its line.
            body += bytes([0x83]) + struct.pack("<hh", x * 20, baseline)
        body += bytes([glyphs]) + bits(*[(0, 1), ((size + 5) * 20, 16)] * glyphs)
    return tag(11, body + bytes([0]))


def define_sprite(cid, *tags):
    """DefineSprite with one frame."""
    return tag(39, struct.pack("<HH", cid, 1) + b"".join(tags) + tag(1) + tag(0))


def place(depth, cid=None, x=0, y=0, name=None, move=False):
    """PlaceObject2: put character cid at depth, or with move=True move what is there."""
    flags = 0x04 | (0x02 if cid else 0) | (0x20 if name else 0) | (0x01 if move else 0)
    body = struct.pack("<BH", flags, depth) + (struct.pack("<H", cid) if cid else b"")
    return tag(26, body + matrix(x, y) + (name.encode() + b"\0" if name else b""))


def place_with_drop_shadow(depth, cid, rgb, distance):
    """PlaceObject3 with a DropShadowFilter pointing straight up, no blur, one pass."""
    # Filter 0 (drop shadow): RGBA, blur x and y, angle and distance (16.16 fixed),
    # strength (8.8 fixed), then flags: composite source, 1 pass.
    shadow = struct.pack("<B4BiiiihB", 0, *rgb, 255, 0, 0, round(-1.5707963 * 65536),
                         distance * 65536, 256, 0x20 | 1)
    flags = struct.pack("<BB", 0x06, 0x01)  # has character and matrix; has filters
    return tag(70, flags + struct.pack("<HH", depth, cid) + matrix() + bytes([1]) + shadow)


# AVM1 bytecode. A call pushes its arguments last first, then their count, the
# object and the method name.
def push(*values):
    data = b""
    for v in values:
        if isinstance(v, bool):
            data += bytes([5, v])
        elif isinstance(v, int):
            data += b"\x07" + struct.pack("<i", v)
        else:
            data += b"\x00" + v.encode() + b"\0"
    return b"\x96" + struct.pack("<H", len(data)) + data


GET_VARIABLE, SET_VARIABLE, GET_MEMBER = b"\x1c", b"\x1d", b"\x4e"
CALL_METHOD, NEW_METHOD, POP, STOP = b"\x52", b"\x53", b"\x17", b"\x07"

actions = (
    # bd = new flash.display.BitmapData(105, 60, false, 0xFFFF00);
    push("bd", 0xFFFF00, False, 60, 105, 4, "flash") + GET_VARIABLE
    + push("display") + GET_MEMBER + push("BitmapData") + NEW_METHOD + SET_VARIABLE
    # bd.draw(offstage);
    + push("offstage") + GET_VARIABLE + push(1, "bd") + GET_VARIABLE
    + push("draw") + CALL_METHOD + POP
    # holder.attachBitmap(bd, 1);
    + push(1, "bd") + GET_VARIABLE + push(2, "holder") + GET_VARIABLE
    + push("attachBitmap") + CALL_METHOD + POP)

frame1 = [
    tag(69, struct.pack("<I", 0)),  # FileAttributes: AVM1
    tag(9, bytes([255, 255, 255])),  # white background
    define_font(),
    # Corner markers: a 4x4 glyph at (0,0) and at (396,296).
    define_text(2, 0, 0, MAGENTA, size=4, glyphs=1), place(1, 2), place(2, 2, 396, 296),
    # Correct bounds crossing the left, top, right and bottom edge: drawn, cut by the edge.
    define_text(10, -20, 20, BLACK), place(10, 10),
    define_text(11, 150, -20, BLACK), place(11, 11),
    define_text(12, 335, 20, BLACK), place(12, 12),
    define_text(13, 150, 280, BLACK), place(13, 13),
    # Bounds wholly off the stage, glyphs on it: Flash Player draws nothing.
    define_text(14, 20, 110, RED, bounds=(-1000, -1000, -900, -900)), place(14, 14),
    # Bounds on the stage but tiny, inside the first glyph: both glyphs drawn in full.
    define_text(15, 150, 110, BLUE, bounds=(160, 120, 170, 130)), place(15, 15),
    # Sprite "offstage" at (1000,1000) holds text at (10,10). The actions draw it
    # into a yellow bitmap shown by sprite "holder" at (270,100); the text shows.
    define_text(16, 10, 10, BLUE), define_sprite(17, place(1, 16)),
    place(16, 17, 1000, 1000, name="offstage"),
    define_sprite(18), place(17, 18, 270, 100, name="holder"),
    # A sprite holds text at x -480: off the stage on frames 1 and 2. Frame 3
    # moves the sprite 500 px right, so the text is drawn at x 20.
    define_text(19, -480, 200, GREEN), define_sprite(20, place(1, 19)), place(18, 20),
    # Text below the stage (y 320) with a cyan drop shadow 120 px up: the shadow is drawn.
    define_text(21, 150, 320, BLACK), place_with_drop_shadow(19, 21, CYAN, 120),
    # Three lines of 20 px squares, the bounds holding only the first line: all
    # three lines are drawn.
    define_text(22, 260, 180, BLUE, bounds=(260, 180, 305, 200), size=20, lines=3), place(20, 22),
    # The same, the bounds holding only the middle of the second line: all three
    # lines are drawn, the first included.
    define_text(23, 330, 180, BLUE, bounds=(330, 215, 375, 225), size=20, lines=3), place(21, 23),
    tag(12, actions + b"\0"),  # DoAction
]
frame3 = [place(18, x=500, move=True), tag(12, STOP + b"\0")]

tags = b"".join(t + tag(1) for t in (b"".join(frame1), b"", b"".join(frame3))) + tag(0)
body = rect(0, 0, 400, 300) + struct.pack("<HH", 24 << 8, 3) + tags  # 24 fps, 3 frames
swf = b"FWS" + bytes([8]) + struct.pack("<I", 8 + len(body)) + body
Path(__file__).with_name("test.swf").write_bytes(swf)
