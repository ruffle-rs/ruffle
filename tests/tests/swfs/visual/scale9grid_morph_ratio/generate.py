#!/usr/bin/env python3

# The morph tag rides along byte for byte from ../scale9grid_morph/generate.py; the movie
# is AVM1 so a two-byte DoAction can stop it on frame 2. See test.toml.

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "scale9grid_morph"))
from generate import (  # noqa: E402
    GRID_TAG_PAYLOAD,
    MORPH_TAG,
    matrix,
    parse_rect,
    place,
    rect,
    tag,
)

GRIDDED_ID = 75
RATIO = 32768
SCALE_X = 3.0
MARGIN = 400


def move_ratio(depth, ratio):
    # PlaceObject2 MOVE|HAS_RATIO: same depth, new ratio, everything else kept.
    body = struct.pack("<BH", 0x01 | 0x10, depth) + struct.pack("<H", ratio)
    return tag(26, body)


def main():
    start, nb = parse_rect(MORPH_TAG[8:])
    end, _ = parse_rect(MORPH_TAG[8 + nb :])
    x0, x1 = min(start[0], end[0]), max(start[1], end[1])
    y0, y1 = min(start[2], end[2]), max(start[3], end[3])

    # Whole-pixel cell pitch, so the two columns rasterise on the same subpixel phase.
    cell_w = -(-(int((x1 - x0) * SCALE_X) + MARGIN) // 20) * 20
    cell_h = (y1 - y0) + MARGIN
    stage_w = -(-(MARGIN + 2 * cell_w) // 20) * 20
    stage_h = -(-(MARGIN + cell_h) // 20) * 20

    out = [tag(9, bytes((0x80, 0x80, 0x80)))]
    out.append(MORPH_TAG)
    out.append(tag(78, GRID_TAG_PAYLOAD))
    for col, ratio in enumerate((0, RATIO)):
        tx = MARGIN - int(x0 * SCALE_X) + col * cell_w
        out.append(place(col + 1, GRIDDED_ID, matrix(SCALE_X, 1.0, tx, MARGIN - y0), ratio))
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
