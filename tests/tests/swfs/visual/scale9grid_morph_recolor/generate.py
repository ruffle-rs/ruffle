#!/usr/bin/env python3

# A morph whose start and end frames share their bounds exactly: a 60x60 square whose
# fill morphs red to blue in place. A ratio change then moves nothing a sliced
# tessellation's cache key can see except the ratio itself, so a tessellation served
# across ratios renders the stale colour. Columns as in ../scale9grid_morph_ratio:
# moved to the new ratio on frame 2, reference placed there from the start.

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "scale9grid_morph"))
from generate import matrix, place, rect, tag  # noqa: E402

MORPH_ID = 80
RATIO = 32768
SCALE_X = 3.0
MARGIN = 400
SIDE = 1200  # twips


class Bits:
    def __init__(self):
        self.s = ""

    def put(self, value, nbits):
        self.s += format(value & ((1 << nbits) - 1), f"0{nbits}b")

    def bytes(self):
        s = self.s + "0" * (-len(self.s) % 8)
        return bytes(int(s[k : k + 8], 2) for k in range(0, len(s), 8))


def move_ratio(depth, ratio):
    # PlaceObject2 MOVE|HAS_RATIO: same depth, new ratio, everything else kept.
    body = struct.pack("<BH", 0x01 | 0x10, depth) + struct.pack("<H", ratio)
    return tag(26, body)


def square_edges(start):
    # Move to origin, four straight edges clockwise, end record. The start shape
    # carries the style bits and selects fill 0 -- the player calls the same tag with
    # fill 1 corrupted -- and the end shape pairs with it move for move and edge for
    # edge, styleless, the way Animate writes morphs.
    b = Bits()
    b.put(0x10 if start else 0x00, 8)
    b.put(0b000011 if start else 0b000001, 6)
    b.put(5, 5)  # move bits
    b.put(0, 5)
    b.put(0, 5)
    if start:
        b.put(1, 1)  # fill style 0
    for dx, dy in ((SIDE, 0), (0, SIDE), (-SIDE, 0), (0, -SIDE)):
        b.put(0b11, 2)  # straight edge
        b.put(10, 4)  # 12-bit deltas
        if dy == 0:
            b.put(0b00, 2)
            b.put(dx, 12)
        else:
            b.put(0b01, 2)
            b.put(dy, 12)
    b.put(0, 6)
    return b.bytes()


def morph_tag():
    bounds = rect(0, SIDE, 0, SIDE)
    fills = bytes((1, 0, 255, 0, 0, 255, 0, 0, 255, 255))  # one solid, red -> blue
    lines = bytes((0,))
    start_edges = square_edges(True)
    body = struct.pack("<H", MORPH_ID) + bounds + bounds
    body += struct.pack("<I", len(fills) + len(lines) + len(start_edges))
    body += fills + lines + start_edges + square_edges(False)
    return tag(46, body)


def main():
    cell_w = -(-(int(SIDE * SCALE_X) + MARGIN) // 20) * 20
    stage_w = -(-(MARGIN + 2 * cell_w) // 20) * 20
    stage_h = -(-(MARGIN + SIDE + MARGIN) // 20) * 20

    out = [tag(9, bytes((0x80, 0x80, 0x80)))]
    out.append(morph_tag())
    out.append(tag(78, struct.pack("<H", MORPH_ID) + rect(400, 800, 400, 800)))
    for col, ratio in enumerate((0, RATIO)):
        out.append(place(col + 1, MORPH_ID, matrix(SCALE_X, 1.0, MARGIN + col * cell_w, MARGIN), ratio))
    out.append(tag(1, b""))
    out.append(move_ratio(1, RATIO))
    out.append(tag(12, bytes((0x07, 0x00))))  # stop()
    out.append(tag(1, b""))
    out.append(tag(0, b""))

    payload = rect(0, stage_w, 0, stage_h) + struct.pack("<HH", 30 << 8, 2)
    payload += b"".join(out)
    data = b"FWS" + bytes([17]) + struct.pack("<I", 8 + len(payload)) + payload
    Path(__file__).with_name("test.swf").write_bytes(data)
    print(f"wrote test.swf ({len(data)} bytes), stage {stage_w / 20:.0f}x{stage_h / 20:.0f}px")


if __name__ == "__main__":
    main()
