// Regression test for #24337.
// Flash tells two failures apart: an end-tag that closes the wrong element
// leaves the element it was nested in unclosed, which is -9 ("a start-tag was
// not matched with an end-tag"), while an end-tag with no start-tag at all is
// -10. Ruffle used to report -10 for both.

function check(source) {
    var doc = new XML(source);
    trace(source + " -> " + doc.status);
}

check("<a><b>foo</a>");
check("foo</a>");
check("<a><b>foo</b></a>");
check("<a>foo</a>");
