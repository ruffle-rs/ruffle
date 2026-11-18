var cases = [
    // End-tag closes the element directly above the open one.
    "<a><b>foo</a>",
    "<a><b>foo</a><c/>",
    "<a><b>foo</a>bar</a>",
    "<a><b>foo</a></b>",
    "<r><a><b>foo</a></r>",
    "<a><b><c>foo</b>",
    // End-tag closes an element further up, or one that isn't open.
    "<a><b><c>foo</a>",
    "<r><a><b><c>foo</a></r>",
    "<a><b>foo</b></c></a>",
    "<a>foo</b>",
    "<a><b/>foo</c>",
    "<a><b>foo</A>",
    // End-tag with nothing open.
    "foo</a>",
    "<a></a></a>",
    // Elements still open at the end of the input.
    "<a>foo",
    "<a><b>foo</b>",
    "<a><b>",
    "<a/><b>",
    // Well-formed.
    "<a><b>foo</b></a>",
    "<a>foo</a>",
    "<a/><b/>"
];
for (var i = 0; i < cases.length; i++) {
    var xml = new XML(cases[i]);
    trace("case: " + cases[i]);
    trace("  returned: " + xml.status);
    trace("  toString: " + xml.toString());
}
