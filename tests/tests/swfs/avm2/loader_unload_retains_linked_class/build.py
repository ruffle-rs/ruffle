#!/usr/bin/env python3
"""Builds test.swf and child/child.swf from the .as sources next to them.

Compiles each class to ABC with the asc.jar Ruffle uses for playerglobal, and
wraps the ABC in a hand-assembled SWF container. The child additionally gets
a DefineShape (a 100x100 square) placed inside a DefineSprite, with a
SymbolClass tag linking that sprite to the `Child` class.

Usage: build.py <path to playerglobal_import.abc>
"""

import struct
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ASC_JAR = HERE.parents[4] / "tools/asc/asc.jar"

SWF_VERSION = 15
STAGE_W, STAGE_H, FPS = 550, 400, 30


class Bits:
    """Big-endian bit writer, as SWF bit fields require."""

    def __init__(self):
        self.bits = []

    def ub(self, value, nbits):
        for i in range(nbits - 1, -1, -1):
            self.bits.append((value >> i) & 1)

    def sb(self, value, nbits):
        self.ub(value & ((1 << nbits) - 1), nbits)

    def bytes(self):
        bits = self.bits + [0] * (-len(self.bits) % 8)
        return bytes(
            int("".join(map(str, bits[i : i + 8])), 2) for i in range(0, len(bits), 8)
        )


def signed_bits(*values):
    return max(abs(v).bit_length() for v in values) + 1


def rect(xmin, xmax, ymin, ymax):
    nbits = signed_bits(xmin, xmax, ymin, ymax)
    b = Bits()
    b.ub(nbits, 5)
    for v in (xmin, xmax, ymin, ymax):
        b.sb(v, nbits)
    return b.bytes()


def tag(code, body):
    if len(body) < 0x3F:
        return struct.pack("<H", (code << 6) | len(body)) + body
    return struct.pack("<HI", (code << 6) | 0x3F, len(body)) + body


def define_shape(shape_id, size_twips, rgb):
    """DefineShape: one solid fill, one square of `size_twips` a side."""
    body = struct.pack("<H", shape_id) + rect(0, size_twips, 0, size_twips)
    body += bytes([1, 0x00]) + bytes(rgb)  # one fill style: solid colour
    body += bytes([0])  # no line styles
    b = Bits()
    b.ub(1, 4)  # NumFillBits
    b.ub(0, 4)  # NumLineBits
    # Style change: move to (0, 0), use fill style 0.
    b.ub(0, 1)  # not an edge
    b.ub(0b00011, 5)  # new styles, line, fill1, fill0=1, move=1
    b.ub(1, 5)  # MoveBits
    b.sb(0, 1)
    b.sb(0, 1)
    b.ub(1, 1)  # FillStyle0 = 1
    nbits = signed_bits(size_twips)
    for dx, dy in ((size_twips, 0), (0, size_twips), (-size_twips, 0), (0, -size_twips)):
        b.ub(1, 1)  # edge
        b.ub(1, 1)  # straight
        b.ub(nbits - 2, 4)
        b.ub(1, 1)  # general line
        b.sb(dx, nbits)
        b.sb(dy, nbits)
    b.ub(0, 6)  # end of shape
    return tag(2, body + b.bytes())


def define_sprite(sprite_id, shape_id):
    place = tag(26, bytes([0x02]) + struct.pack("<HH", 1, shape_id))  # PlaceObject2
    return tag(39, struct.pack("<HH", sprite_id, 1) + place + tag(1, b"") + tag(0, b""))


def symbol_class(links):
    body = struct.pack("<H", len(links))
    for character_id, name in links:
        body += struct.pack("<H", character_id) + name.encode() + b"\x00"
    return tag(76, body)


def do_abc(abc, lazy):
    return tag(82, struct.pack("<I", 1 if lazy else 0) + b"\x00" + abc)


def swf(tags):
    body = rect(0, STAGE_W * 20, 0, STAGE_H * 20)
    body += struct.pack("<HH", FPS << 8, 1)
    body += tag(69, struct.pack("<I", 0x08))  # FileAttributes: ActionScript 3
    body += b"".join(tags)
    body += tag(1, b"") + tag(0, b"")
    header = b"FWS" + bytes([SWF_VERSION])
    return header + struct.pack("<I", len(header) + 4 + len(body)) + body


def compile_abc(source, playerglobal):
    with tempfile.TemporaryDirectory() as tmp:
        subprocess.run(
            [
                "java",
                "-classpath",
                str(ASC_JAR),
                "macromedia.asc.embedding.ScriptCompiler",
                "-optimize",
                "-import",
                str(playerglobal),
                "-outdir",
                tmp,
                "-out",
                source.stem,
                str(source),
            ],
            check=True,
            capture_output=True,
            text=True,
        )
        return (Path(tmp) / f"{source.stem}.abc").read_bytes()


def main():
    playerglobal = Path(sys.argv[1])

    test_abc = compile_abc(HERE / "Test.as", playerglobal)
    (HERE / "test.swf").write_bytes(
        swf([do_abc(test_abc, lazy=True), symbol_class([(0, "Test")])])
    )

    child_abc = compile_abc(HERE / "child/Child.as", playerglobal)
    (HERE / "child/child.swf").write_bytes(
        swf(
            [
                define_shape(1, 100 * 20, (0x40, 0x80, 0xC0)),
                define_sprite(2, 1),
                do_abc(child_abc, lazy=False),
                symbol_class([(2, "Child")]),
            ]
        )
    )


if __name__ == "__main__":
    main()
