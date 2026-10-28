var big = new TextFormat();
big.size = 30;
var mid = new TextFormat();
mid.size = 20;
var huge = new TextFormat();
huge.size = 40;
var depth = 1;

function show(label) {
    var sizes = [];
    for (var i = 0; i < field.length; ++i) {
        sizes.push(field.getTextFormat(i).size);
    }
    trace("  " + label + ": text=" + field.text + " sizes=" + sizes.join(",")
        + " new=" + field.getNewTextFormat().size);
}

function setup() {
    _root.createTextField("field" + depth, depth, 0, 0, 200, 40);
    field = _root["field" + depth];
    depth++;
    field.text = "AB";
    show("text = \"AB\"");
    field.setTextFormat(big);
    show("setTextFormat(big)");
}

function finish() {
    field.setTextFormat(huge);
    show("setTextFormat(huge)");
    field.text = field.text;
    show("text = field.text");
}

var ops = [
    ["replaceText(0, 1, \"A\")", function() { field.replaceText(0, 1, "A"); }],
    ["replaceText(2, 2, \"\")", function() { field.replaceText(2, 2, ""); }],
    ["replaceSel(\"\")", function() { field.replaceSel(""); }],
    ["replaceSel(\"X\")", function() { field.replaceSel("X"); }]
];

for (var k = 0; k < ops.length; ++k) {
    var name = ops[k][0];
    var op = ops[k][1];
    trace("// " + name + ", then same text");
    setup();
    op();
    show(name);
    finish();

    trace("// setNewTextFormat, " + name + ", then same text");
    setup();
    field.setNewTextFormat(mid);
    show("setNewTextFormat(mid)");
    op();
    show(name);
    finish();

    trace("// " + name + ", setNewTextFormat, then same text");
    setup();
    op();
    show(name);
    field.setNewTextFormat(mid);
    show("setNewTextFormat(mid)");
    finish();
}

trace("// replaceText(0, 1, \"X\"), then old text");
setup();
field.replaceText(0, 1, "X");
show("replaceText(0, 1, \"X\")");
field.text = "AB";
show("text = \"AB\"");

trace("// replaceSel(\"X\"), then old text");
setup();
field.replaceSel("X");
show("replaceSel(\"X\")");
field.text = "AB";
show("text = \"AB\"");

trace("// replaceSel(\"X\"), then same text twice");
setup();
field.replaceSel("X");
show("replaceSel(\"X\")");
finish();
finish();
