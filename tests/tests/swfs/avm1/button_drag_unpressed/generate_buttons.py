"""Generate native button art: idle width 80, hover width 60, pressed width 40.

Run with Python 3 and an optional output path; scripts are compiled by Rascal.
"""
from pathlib import Path
import struct
import sys


class Bits:
    def __init__(self):
        self.bits = ""

    def put(self, value, count):
        self.bits += format(value & ((1 << count) - 1), f"0{count}b")

    def bytes(self):
        b = self.bits + "0" * (-len(self.bits) % 8)
        return bytes(int(b[i:i + 8], 2) for i in range(0, len(b), 8))


def tag(code, data=b""):
    return struct.pack("<H", code << 6 | min(63, len(data))) + (struct.pack("<I", len(data)) if len(data) >= 63 else b"") + data


def rect(xmin, xmax, ymin, ymax):
    bits = Bits()
    n = max(abs(v).bit_length() + 1 for v in [xmin, xmax, ymin, ymax])
    bits.put(n, 5)
    for v in [xmin, xmax, ymin, ymax]:
        bits.put(v, n)
    return bits.bytes()


def shape(character, color, width=1600):
    bits = Bits()
    bits.put(1, 4)  # One fill index bit.
    bits.put(0, 4)  # No line index bits.
    bits.put(0, 1)
    bits.put(0b00101, 5)  # MoveTo and FillStyle1.
    bits.put(1, 5)
    bits.put(0, 1)
    bits.put(0, 1)
    bits.put(1, 1)
    for dx, dy in [(width, 0), (0, 1600), (-width, 0), (0, -1600)]:
        n = max(abs(dx).bit_length(), abs(dy).bit_length()) + 1
        bits.put(1, 1)
        bits.put(1, 1)
        bits.put(n - 2, 4)
        bits.put(0, 1)  # Axis-aligned edge.
        bits.put(dx == 0, 1)
        bits.put(dy if dx == 0 else dx, n)
    bits.put(0, 6)
    data = struct.pack("<H", character) + rect(0, width, 0, 1600)
    data += b"\x01\x00" + bytes.fromhex(color) + b"\x00" + bits.bytes()
    return tag(2, data)


art = shape(1000, "6699cc") + shape(1001, "ffffff", 1200) + shape(1002, "ff9900", 800)
records = b""
for flags, character in [(9, 1000), (2, 1001), (4, 1002)]:
    records += bytes([flags]) + struct.pack("<HH", character, 1) + b"\x00\x00"
art += tag(34, struct.pack("<HBH", 1003, 0, 0) + records + b"\x00")
frames = b""
for label, character in [("_up", 1000), ("_over", 1001), ("_down", 1002)]:
    frames += tag(43, label.encode() + b"\x00")
    if character != 1000:
        frames += tag(28, struct.pack("<H", 1))
    frames += tag(26, b"\x06" + struct.pack("<HH", 1, character) + b"\x00")
    frames += tag(12, b"\x07\x00") + tag(1)
art += tag(39, struct.pack("<HH", 1004, 3) + frames + tag(0))
art += tag(56, struct.pack("<HH", 2, 1003) + b"tile\x00" + struct.pack("<H", 1004) + b"movie_tile\x00")
body = rect(0, 6800, 0, 8000) + struct.pack("<HH", 24 * 256, 1)
body += tag(69, b"\x00" * 4) + tag(9, b"\xff" * 3) + art + tag(1) + tag(0)
(Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("buttons.swf")).write_bytes(
    b"FWS\x08" + struct.pack("<I", len(body) + 8) + body
)
