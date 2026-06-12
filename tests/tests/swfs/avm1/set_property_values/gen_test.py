#!/usr/bin/env python3

# Properties that can be set via setProperty().
properties = [
    "_x",
    "_y",
    "_xscale",
    "_yscale",
    "_alpha",
    "_visible",
    "_width",
    "_height",
    "_rotation",
    "_name",
    "_highquality",
    "_focusrect",
    "_soundbuftime",
    "_quality",
]

# Values to try assigning to each property.
values = [
    "2",
    "-5",
    "1.5",
    "0",
    '"10"',
    '"10x"',
    '"z10"',
    '"abc"',
    '""',
    '"true"',
    '"false"',
    '"undefined"',
    '"null"',
    "true",
    "false",
    "undefined",
    "null",
    "0.0/0.0",
    "1.0/0.0",
    "-1.0/0.0",
    "new Object()",
]

for prop in properties:
    for value in values:
        print(f'trace("// setProperty({prop}, {value.replace('"', '\\"')})");')

        print(f'setProperty("_level0", {prop}, 0);')
        print(f'setProperty("_level0", {prop}, {value});')
        print(f'trace(getProperty("_level0", {prop}));')

        print(f'setProperty("_level0", {prop}, 1);')
        print(f'setProperty("_level0", {prop}, {value});')

        print(f'trace(getProperty("_level0", {prop}));')
        print(f'setProperty("_level0", {prop}, 0);')
        print('obj = new Object();')
        print('obj.valueOf = function() { trace("valueOf"); return ' + value + '; };')
        print('obj.toString = function() { trace("toString"); return ' + value + '; };')
        print(f'setProperty("_level0", {prop}, obj);')
        print(f'trace(getProperty("_level0", {prop}));')

        print('trace("----------");')
