#!/usr/bin/env python3
r"""Build the SWFs of this test.

    python3 make_swfs.py  # writes test.swf, child6.swf, child7.swf

child6.swf (SWF 6) and child7.swf (SWF 7) are the same movie except for the
version in the SWF header. Each defines three fonts and five text fields:

* "STFont" (id 1) is a DefineFont2 font with the SmallText flag set (font
  flags 0xa4), the flag the Flash authoring tool sets for "Bitmap text
  [no anti-alias]".
* "PlainFont" (id 2) is the same font without the flag (font flags 0x84).
* "ST3Font" (id 3) is a DefineFont3 copy of "STFont", also with the flag.

Every glyph is a solid bar. Advances are given in a 1024 unit em (stored at
20x in DefineFont3) and were picked so that they fall between whole pixels at
the sizes used here.

test.swf (SWF 8) loads both children and traces the textWidth of every field
once they are loaded.
"""
import os
import struct

# (character, advance in 1/1024 em)
GLYPHS = [(" ", 256), ("a", 440), ("b", 512), ("c", 257)]
ASCENT, DESCENT = 900, 200
BAR_WIDTH, BAR_HEIGHT = 154, 716

# (instance name, font id, size in pixels, text, description)
FIELDS = [
    ("f1", 1, 10, "aaaaaaaaaa", "SmallText DefineFont2 10px"),
    ("f2", 1, 20, "abcabcabc", "SmallText DefineFont2 20px"),
    ("f3", 2, 10, "aaaaaaaaaa", "plain DefineFont2 10px"),
    ("f4", 3, 10, "aaaaaaaaaa", "SmallText DefineFont3 10px"),
    ("f5", 3, 20, "abcabcabc", "SmallText DefineFont3 20px"),
]


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


def matrix(tx, ty):
    w = BitWriter()
    w.bits(0, 1)  # HasScale
    w.bits(0, 1)  # HasRotate
    n = signed_bits(tx, ty)
    w.bits(n, 5)
    w.bits(tx & ((1 << n) - 1), n)
    w.bits(ty & ((1 << n) - 1), n)
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


def define_font(font_id, name, flags, version):
    """A DefineFont2 (version 2) or DefineFont3 (version 3) tag."""
    scale = 20 if version == 3 else 1
    shapes = [bar_shape(0 if c == " " else BAR_WIDTH * scale, BAR_HEIGHT * scale) for c, _ in GLYPHS]
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
    body += struct.pack("<HHh", ASCENT * scale, DESCENT * scale, 0)
    body += b"".join(struct.pack("<h", a * scale) for _, a in GLYPHS)
    body += b"".join(
        rect(0, 0 if c == " " else BAR_WIDTH * scale, -BAR_HEIGHT * scale, 0) for c, _ in GLYPHS
    )
    body += struct.pack("<H", 0)  # KerningCount
    return tag(75 if version == 3 else 48, body)


def define_edit_text(character_id, font_id, size, text):
    # HasText, ReadOnly, HasTextColor, HasFont; NoSelect, UseOutlines.
    flags = bytes([0x80 | 0x08 | 0x04 | 0x01, 0x10 | 0x01])
    body = struct.pack("<H", character_id) + rect(0, 380 * 20, 0, (size + 12) * 20) + flags
    body += struct.pack("<HH", font_id, size * 20) + bytes([0, 0, 0, 255])
    body += b"\0" + text.encode("ascii") + b"\0"  # VariableName, InitialText
    return tag(37, body)


def place_object_2(depth, character_id, name, y):
    body = bytes([0x02 | 0x04 | 0x20]) + struct.pack("<HH", depth, character_id)
    body += matrix(20 * 20, y * 20) + name.encode("ascii") + b"\0"
    return tag(26, body)


def swf(version, frame_count, tags):
    body = rect(0, 400 * 20, 0, 300 * 20) + struct.pack("<HH", 24 << 8, frame_count) + tags
    return b"FWS" + bytes([version]) + struct.pack("<I", 8 + len(body)) + body


SHOW_FRAME = struct.pack("<H", 1 << 6)
END = struct.pack("<H", 0)


def child(version):
    tags = tag(9, bytes([255, 255, 255]))  # SetBackgroundColor
    tags += define_font(1, "STFont", 0xA4, 2)
    tags += define_font(2, "PlainFont", 0x84, 2)
    tags += define_font(3, "ST3Font", 0xA4, 3)
    y = 10
    for i, (name, font_id, size, text, _) in enumerate(FIELDS):
        tags += define_edit_text(100 + i, font_id, size, text)
        tags += place_object_2(1 + i, 100 + i, name, y)
        y += size + 20
    return swf(version, 1, tags + SHOW_FRAME + END)


# A tiny AVM1 assembler.
def push(*values):
    data = b""
    for v in values:
        if v is None:
            data += b"\x03"  # undefined
        elif isinstance(v, str):
            data += b"\x00" + v.encode("ascii") + b"\0"
        else:
            data += b"\x07" + struct.pack("<i", v)
    return b"\x96" + struct.pack("<H", len(data)) + data


GET_VARIABLE, SET_VARIABLE, GET_MEMBER, CALL_METHOD = b"\x1c", b"\x1d", b"\x4e", b"\x52"
POP, ADD2, EQUALS2, OR, TRACE, PLAY, STOP = b"\x17", b"\x47", b"\x49", b"\x11", b"\x26", b"\x06", b"\x07"


def get(path):
    """Push the value of a dotted path such as "v6.f1.textWidth"."""
    first, *rest = path.split(".")
    code = push(first) + GET_VARIABLE
    for member in rest:
        code += push(member) + GET_MEMBER
    return code


def call_method(obj, method, args):
    code = b"".join(reversed(args)) + push(len(args)) + obj + push(method) + CALL_METHOD
    return code + POP


def branch(opcode, offset):
    return opcode + struct.pack("<Hh", 2, offset)


def goto_frame(frame):
    return b"\x81" + struct.pack("<HH", 2, frame)


def do_action(code):
    return tag(12, code + b"\0")


def output(text_code):
    return push("s") + text_code + SET_VARIABLE + get("s") + TRACE


def parent():
    # Frame 1: load the children.
    load = b""
    for depth, version in enumerate((6, 7), start=1):
        clip = f"v{version}"
        load += call_method(get("_root"), "createEmptyMovieClip", [push(clip), push(depth)])
        load += call_method(get(clip), "loadMovie", [push(f"child{version}.swf")])

    # Frame 3: go back to frame 2 until both children are loaded, then trace.
    report = b""
    for version in (6, 7):
        for name, _, _, _, description in FIELDS:
            label = push(f"SWF {version} {description}: textWidth ")
            report += output(label + get(f"v{version}.{name}.textWidth") + ADD2)
    report += STOP
    not_loaded = goto_frame(1) + PLAY
    loaded = report + branch(b"\x99", len(not_loaded))
    check = get("v6.f1") + push(None) + EQUALS2 + get("v7.f1") + push(None) + EQUALS2 + OR
    check += branch(b"\x9d", len(loaded))
    wait = check + loaded + not_loaded

    tags = tag(69, struct.pack("<I", 0))  # FileAttributes
    tags += tag(9, bytes([255, 255, 255]))
    tags += do_action(load) + SHOW_FRAME
    tags += SHOW_FRAME
    tags += do_action(wait) + SHOW_FRAME
    return swf(8, 3, tags + END)


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    targets = {
        os.path.join(here, "test.swf"): parent(),
        os.path.join(here, "child6.swf"): child(6),
        os.path.join(here, "child7.swf"): child(7),
    }
    for path, data in targets.items():
        with open(path, "wb") as f:
            f.write(data)


if __name__ == "__main__":
    main()
