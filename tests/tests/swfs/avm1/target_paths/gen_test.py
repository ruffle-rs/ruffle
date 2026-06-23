#!/usr/bin/env python3

# The following paths have been selected as interesting cases from a bruteforce
# iteration over various paths.
paths = [
    "/",
    "//",
    "/a",
    "//a",
    "/",
    "/a",
    "/a/",
    "a",
    "a/",
    "a/..",
    "a/../",
    "a/..:",
    "a/../a",
    "a/..:a",
    "../a",
    "a/a/..",
    "a/a/../",
    "a/a/../..",
    "a/a/../../",
    "a/a/../../a",

    "variable",
    "/:variable",
    ":variable",
    "/a:variable",
    "a:variable",
    "/a/:variable",
    "a/:variable",
    "a/../a:variable",
    "/:root_var",
    "root_var",
    ":root_var",
    "/a:root_var",
    "a:root_var",
    "/a/:root_var",
    "a/:root_var",
    "a/../a:root_var",
    "/:obj_var",

    "/.variable",
    ".variable",
    "/a.variable",
    "a.variable",
    "/a/.variable",
    "a/.variable",
    "a/../a.variable",
    "/.root_var",
    ".root_var",
    "/a.root_var",
    "a.root_var",
    "/a/.root_var",
    "a/.root_var",
    "a/../a.root_var",

    "_level0:root_var",
    "_level0:variable",
    "_level0/:root_var",
    "_level0/:variable",

    "_level0.root_var",
    "_level0.variable",
    "_level0/.root_var",
    "_level0/.variable",

    "a/child",
    "a:child",
    "a.child",
    "_level0.a/child",
    "_level0/a.child",
    "_level0:a:child",
    "_level0.a:child",
    "_level0:a.child",
    "_level0:a/child",
    "_level0/a:child",

    "/_level0",
    "_level0",
    "_level0/_level0",
    "unknown/_level0",
    "_level0/a:_level0",
    "_level0/a",
    "_level0/../a",
    "_level0/../_level0",
    "_level0/:a/",
    "_level0/.a/",
    "_level0/:a/_level0",
    "_level0/:a:_level0",
    "_level0/:a:",
    "_level0/.a:",
    "_level0/:a",
    "_level0/.a",
    "_level0/::a",
    "_level0/:::a",
    "_level0/:::_level0",
    "_level0/::_level0",
    "_level0/:_level0",

    "/this",
    "this",
    "/_root",
    "_root",
    "_root/a",
    "/_level0/a",

    "_flash0",
    "_flash0/",
    "/_flash0",
    "_flash0/a",

    "_flash0...:",
    "_level0...",
    ":_flash0:/",
    "_level0:_level0:/",
    "_flash0..:",
    "::_flash0..",
    "_root/_level0:/",
    "a:_level0..",
    "_level0..:_flash0",
    "_level0..:/",
    "_root:_flash0..",
    "_flash0..",
    "_flash0/_flash0:/",
    "_level0..:_flash0",
    "/:_level0:/",
    "a/_flash0:/",
]

print('var root_var = 5;')
print('var obj_var = {};')
print('trace(a);')
print('trace(b);')
print('trace(child);')
print('trace(root_var);')
print('trace(typeof root_var);')
print('trace(typeof obj_var);')

for path in paths:
    print(f'trace("path {path}");')
    print(f'trace("  _x:" add getProperty("{path}", _x));')
    print(f'trace("  _name:" add getProperty("{path}", _name));')
    print(f'trace("  typeof eval:" add (typeof eval("{path}")));')
    print(f'tellTarget("{path}") ' + '{')
    print(f'  trace("  tellTarget");')
    print(f'  trace("  _x:" add _x);')
    print(f'  trace("  _name:" add _name);')
    print('}')
