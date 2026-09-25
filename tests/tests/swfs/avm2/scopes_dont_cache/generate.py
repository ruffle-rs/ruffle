#!/usr/bin/env python3

# NOTE: This script will generate only the SWF file for the `order-1` test. To
# generate the SWF file for `order-2`, swap line 210 with line 213.

"""Generate an ABC that tests ordering in the captured ScopeChain cache.

The lookup name is passed as a function argument and consumed by MultinameL
instructions so the verifier cannot replace the lookups with static scope
accesses.
"""

import struct

def u30(value):
    result = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            result.append(byte | 0x80)
        else:
            result.append(byte)
            return bytes(result)


def string(value):
    encoded = value.encode("utf-8")
    return u30(len(encoded)) + encoded


def method(param_count=0):
    return (
        u30(param_count)
        + u30(0)  # return_type: *
        + b"".join(u30(0) for _ in range(param_count))
        + u30(0)  # name
        + b"\x00"  # flags
    )


def slot_trait(name, slot_id, value_index):
    return (
        u30(name)
        + b"\x00"
        + u30(slot_id)
        + u30(0)  # type_name: *
        + u30(value_index)
        + b"\x01"  # ConstantKind::Utf8
    )


def method_body(
    method_index,
    max_stack,
    local_count,
    init_scope_depth,
    max_scope_depth,
    code,
):
    return b"".join(
        [
            u30(method_index),
            u30(max_stack),
            u30(local_count),
            u30(init_scope_depth),
            u30(max_scope_depth),
            u30(len(code)),
            code,
            u30(0),  # exception_count
            u30(0),  # trait_count
        ]
    )


def op(opcode, *operands):
    return bytes([opcode]) + b"".join(u30(value) for value in operands)


PUSHNULL = b"\x20"
POP = b"\x29"
PUSHSCOPE = b"\x30"
RETURNVOID = b"\x47"
GETLOCAL_0 = b"\xD0"
GETLOCAL_1 = b"\xD1"
POPSCOPE = b"\x1D"

OP_CALL = 0x41
OP_CALLPROPVOID = 0x4F
OP_FINDPROPSTRICT = 0x5D
OP_GETPROPERTY = 0x66
OP_NEWFUNCTION = 0x40
OP_PUSHSTRING = 0x2C

CONSTANT_PACKAGE_NAMESPACE = 0x16
CONSTANT_QNAME = 0x07
CONSTANT_MULTINAME_L = 0x1B


def build_abc():
    STR_EMPTY = 1
    STR_VALUE = 2
    STR_TRACE = 3
    STR_OUTER_NS = 4
    STR_INNER_NS = 5
    STR_OUTER_VALUE = 6
    STR_INNER_VALUE = 7

    NS_PUBLIC = 1
    NS_OUTER = 2
    NS_INNER = 3

    MN_OUTER_VALUE = 1
    MN_INNER_VALUE = 2
    MN_TRACE = 3
    MN_VALUE_OUTER_THEN_INNER = 4
    MN_VALUE_OUTER = 5
    MN_VALUE_INNER = 6

    M_INNER_INIT = 0
    M_OUTER_INIT = 1
    M_LOOKUP = 2

    abc = bytearray(struct.pack("<HH", 16, 46))
    abc += u30(1) + u30(1) + u30(1)

    strings = [
        "",
        "value",
        "trace",
        "outer",
        "inner",
        "outer-value",
        "inner-value",
    ]
    abc += u30(len(strings) + 1)
    for value in strings:
        abc += string(value)

    namespaces = [
        (CONSTANT_PACKAGE_NAMESPACE, STR_EMPTY),
        (CONSTANT_PACKAGE_NAMESPACE, STR_OUTER_NS),
        (CONSTANT_PACKAGE_NAMESPACE, STR_INNER_NS),
    ]
    abc += u30(len(namespaces) + 1)
    for kind, name in namespaces:
        abc += bytes([kind]) + u30(name)

    abc += u30(4)  # namespace_set_count
    abc += u30(2) + u30(NS_OUTER) + u30(NS_INNER)
    abc += u30(1) + u30(NS_OUTER)
    abc += u30(1) + u30(NS_INNER)

    multinames = [
        (CONSTANT_QNAME, [NS_OUTER, STR_VALUE]),
        (CONSTANT_QNAME, [NS_INNER, STR_VALUE]),
        (CONSTANT_QNAME, [NS_PUBLIC, STR_TRACE]),
        (CONSTANT_MULTINAME_L, [1]),
        (CONSTANT_MULTINAME_L, [2]),
        (CONSTANT_MULTINAME_L, [3]),
    ]
    abc += u30(len(multinames) + 1)
    for kind, operands in multinames:
        abc += bytes([kind])
        for operand in operands:
            abc += u30(operand)

    abc += u30(3) + method() + method() + method(1)
    abc += u30(0)  # metadata_count
    abc += u30(0)  # class_count

    abc += u30(2)  # script_count
    abc += (
        u30(M_INNER_INIT)
        + u30(1)
        + slot_trait(MN_INNER_VALUE, 1, STR_INNER_VALUE)
    )
    abc += (
        u30(M_OUTER_INIT)
        + u30(1)
        + slot_trait(MN_OUTER_VALUE, 1, STR_OUTER_VALUE)
    )

    inner_init = GETLOCAL_0 + PUSHSCOPE + RETURNVOID
    # Capture outer::value's global object below inner::value's global object.
    # The nested function will therefore resolve a matching trait on "inner"
    # before checking "outer".
    outer_init = b"".join(
        [
            GETLOCAL_0,
            PUSHSCOPE,
            op(OP_FINDPROPSTRICT, MN_INNER_VALUE),
            PUSHSCOPE,
            op(OP_NEWFUNCTION, M_LOOKUP),
            PUSHNULL,
            op(OP_PUSHSTRING, STR_VALUE),
            op(OP_CALL, 1),
            POP,
            POPSCOPE,
            POPSCOPE,
            RETURNVOID,
        ]
    )

    # Populate the captured ScopeChain cache in lexical-priority order:
    # inner first, then outer. The final broad lookup must still select inner.
    lookup = b"".join(
        [
            GETLOCAL_1,
            op(OP_FINDPROPSTRICT, MN_VALUE_INNER), # SWAP THIS LINE...
            POP,
            GETLOCAL_1,
            op(OP_FINDPROPSTRICT, MN_VALUE_OUTER), # ...WITH THIS ONE
            POP,
            op(OP_FINDPROPSTRICT, MN_TRACE),
            GETLOCAL_1,
            op(OP_FINDPROPSTRICT, MN_VALUE_OUTER_THEN_INNER),
            GETLOCAL_1,
            op(OP_GETPROPERTY, MN_VALUE_OUTER_THEN_INNER),
            op(OP_CALLPROPVOID, MN_TRACE, 1),
            RETURNVOID,
        ]
    )

    abc += u30(3)
    abc += method_body(M_INNER_INIT, 1, 1, 1, 2, inner_init)
    abc += method_body(M_OUTER_INIT, 3, 1, 1, 3, outer_init)
    abc += method_body(M_LOOKUP, 3, 2, 3, 3, lookup)
    return bytes(abc)


def rect(width, height):
    xmax = width * 20
    ymax = height * 20
    nbits = max(xmax.bit_length(), ymax.bit_length(), 1)
    bits = f"{nbits:05b}{0:0{nbits}b}{xmax:0{nbits}b}{0:0{nbits}b}{ymax:0{nbits}b}"
    while len(bits) % 8:
        bits += "0"
    return bytes(int(bits[i : i + 8], 2) for i in range(0, len(bits), 8))


def tag(code, payload):
    if len(payload) < 0x3F:
        return struct.pack("<H", (code << 6) | len(payload)) + payload
    return struct.pack("<HI", (code << 6) | 0x3F, len(payload)) + payload


def build_swf():
    doabc = struct.pack("<I", 0) + b"scope_cache_order\x00" + build_abc()
    body = (
        rect(100, 100)
        + struct.pack("<HH", 24 << 8, 1)
        + tag(69, struct.pack("<I", 8))
        + tag(82, doabc)
        + tag(1, b"")
        + tag(0, b"")
    )
    return b"FWS" + bytes([10]) + struct.pack("<I", len(body) + 8) + body


if __name__ == "__main__":
    with open("cache-order-2.swf", "wb") as output:
        output.write(build_swf())
