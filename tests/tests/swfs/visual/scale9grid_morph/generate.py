#!/usr/bin/env python3

import struct
from pathlib import Path

MORPH_TAG = bytes.fromhex(
    "bf0b9f0000004b006dd410f85e854d006e0f0f2062852c8050000000010000000080000000"
    "8001000000000000000000000000112db750420f4800e08a13a8d2a0fa938f0014b8e000a0"
    "c5752a14578001f948001f95ec57d35f4574720014c710015f7a9535f3a8800e08000005b8"
    "3c420d20038fa3fec94a3eec8cb40052cb00023e1394a4013800070d200070dc01394dc213"
    "7350005334c005c2ecb4dc1ec80038f800"
)
GRIDDED_ID = 75
PLAIN_ID = 76
WRAPPER_ID = 100
GRID_TAG_PAYLOAD = bytes.fromhex("4b0077a121618ef04850")
GRID_RECT = GRID_TAG_PAYLOAD[2:]

RATIOS = [0, 32768, 65535]
SCALE_X = 3.0
MARGIN = 400


def tag(code, payload):
    if len(payload) < 0x3F:
        return struct.pack("<H", (code << 6) | len(payload)) + payload
    return struct.pack("<HI", (code << 6) | 0x3F, len(payload)) + payload


def rect(x0, x1, y0, y1):
    vals = [x0, x1, y0, y1]
    nbits = max(abs(v).bit_length() for v in vals) + 2
    bits = format(nbits, "05b") + "".join(
        format(v & ((1 << nbits) - 1), f"0{nbits}b") for v in vals
    )
    bits += "0" * (-len(bits) % 8)
    return bytes(int(bits[k : k + 8], 2) for k in range(0, len(bits), 8))


def parse_rect(b):
    nbits = b[0] >> 3
    nb = (5 + 4 * nbits + 7) // 8
    bits = "".join(format(x, "08b") for x in b[:nb])
    vals = []
    for k in range(4):
        v = int(bits[5 + k * nbits : 5 + (k + 1) * nbits], 2)
        if v >> (nbits - 1):
            v -= 1 << nbits
        vals.append(v)
    return vals, nb


def signed_bits(vals):
    n = 1
    for v in vals:
        while not (-(1 << (n - 1)) <= v < (1 << (n - 1))):
            n += 1
    return n


def matrix(sx, sy, tx, ty):
    fx, fy = int(sx * 65536), int(sy * 65536)
    ns = signed_bits([fx, fy])
    nt = signed_bits([tx, ty])
    bits = "1" + format(ns, "05b")
    bits += "".join(format(v & ((1 << ns) - 1), f"0{ns}b") for v in (fx, fy))
    bits += "0"
    bits += format(nt, "05b")
    bits += "".join(format(v & ((1 << nt) - 1), f"0{nt}b") for v in (tx, ty))
    bits += "0" * (-len(bits) % 8)
    return bytes(int(bits[k : k + 8], 2) for k in range(0, len(bits), 8))


def place(depth, cid, mtx, ratio=None):
    flags = 0x02 | 0x04 | (0x10 if ratio is not None else 0)
    body = struct.pack("<BH", flags, depth) + struct.pack("<H", cid) + mtx
    if ratio is not None:
        body += struct.pack("<H", ratio)
    return tag(26, body)


def sprite(sprite_id, ratio):
    body = struct.pack("<HH", sprite_id, 1)
    body += place(1, PLAIN_ID, matrix(1.0, 1.0, 0, 0), ratio)
    body += tag(1, b"")
    body += tag(0, b"")
    return tag(39, body)


def main():
    start, nb = parse_rect(MORPH_TAG[8:])
    end, _ = parse_rect(MORPH_TAG[8 + nb :])
    x0, x1 = min(start[0], end[0]), max(start[1], end[1])
    y0, y1 = min(start[2], end[2]), max(start[3], end[3])

    cell_w = int((x1 - x0) * SCALE_X) + MARGIN
    cell_h = (y1 - y0) + MARGIN
    stage_w = -(-(MARGIN + 3 * cell_w) // 20) * 20
    stage_h = -(-(MARGIN + len(RATIOS) * cell_h) // 20) * 20

    plain_tag = tag(46, struct.pack("<H", PLAIN_ID) + MORPH_TAG[8:])

    out = [tag(69, struct.pack("<I", 1 << 3))]
    out.append(tag(9, bytes((0x80, 0x80, 0x80))))
    out.append(MORPH_TAG)
    out.append(plain_tag)
    out.append(tag(78, GRID_TAG_PAYLOAD))

    # Third column: the same morph inside a gridded sprite, so it is sliced by the
    # parent's grid rather than its own. Each ratio needs its own sprite, since the
    # ratio comes from the PlaceObject inside the sprite's timeline.
    for row in range(len(RATIOS)):
        out.append(sprite(WRAPPER_ID + row, RATIOS[row]))
        out.append(tag(78, struct.pack("<H", WRAPPER_ID + row) + GRID_RECT))

    depth = 1
    for row, ratio in enumerate(RATIOS):
        ty = MARGIN - y0 + row * cell_h
        for col, cid in enumerate((GRIDDED_ID, PLAIN_ID, WRAPPER_ID + row)):
            tx = MARGIN - int(x0 * SCALE_X) + col * cell_w
            mtx = matrix(SCALE_X, 1.0, tx, ty)
            # The wrapper sprite carries no ratio; its child morph was placed at one.
            out.append(place(depth, cid, mtx, ratio if col < 2 else None))
            depth += 1
    out.append(tag(1, b""))
    out.append(tag(0, b""))

    payload = rect(0, stage_w, 0, stage_h) + struct.pack("<HH", 30 << 8, 1)
    payload += b"".join(out)
    data = b"FWS" + bytes([17]) + struct.pack("<I", 8 + len(payload)) + payload
    Path(__file__).with_name("test.swf").write_bytes(data)
    print(f"wrote test.swf ({len(data)} bytes), stage {stage_w / 20:.0f}x{stage_h / 20:.0f}px")


if __name__ == "__main__":
    main()
