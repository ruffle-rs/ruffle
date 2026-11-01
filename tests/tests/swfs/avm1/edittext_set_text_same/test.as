var big = new TextFormat();
big.size = 30;
var mid = new TextFormat();
mid.size = 20;
var depth = 1;

function fresh() {
    _root.createTextField("field" + depth, depth, 0, 0, 200, 40);
    field = _root["field" + depth];
    depth++;
}

function show(label) {
    var a = field.getTextFormat(0);
    var b = field.getTextFormat(1);
    var n = field.getNewTextFormat();
    trace("  " + label + ": text=" + field.text + " sizes=" + a.size + "," + b.size + " colors=" + a.color
        + "," + b.color + " new=" + n.size + "/" + n.color);
}

trace("// setTextFormat, then same text");
fresh();
field.text = "AB";
show("text = \"AB\"");
field.setTextFormat(big);
show("setTextFormat(big)");
field.text = "AB";
show("text = \"AB\"");

trace("// setNewTextFormat, then same text");
fresh();
field.text = "AB";
show("text = \"AB\"");
field.setTextFormat(big);
show("setTextFormat(big)");
field.setNewTextFormat(mid);
show("setNewTextFormat(mid)");
field.text = "AB";
show("text = \"AB\"");

trace("// setNewTextFormat, same text twice");
fresh();
field.text = "AB";
show("text = \"AB\"");
field.setNewTextFormat(mid);
show("setNewTextFormat(mid)");
field.text = "AB";
show("text = \"AB\"");
field.setTextFormat(big);
show("setTextFormat(big)");
field.text = "AB";
show("text = \"AB\"");

trace("// textColor, then same text");
fresh();
field.text = "AB";
show("text = \"AB\"");
field.setTextFormat(big);
show("setTextFormat(big)");
field.textColor = 0xFF0000;
show("textColor = 0xFF0000");
field.text = "AB";
show("text = \"AB\"");

trace("// htmlText, then same text");
fresh();
field.html = true;
field.htmlText = "<font size=\"30\">AB</font>";
show("htmlText = \"<font size='30'>AB</font>\"");
field.text = "AB";
show("text = \"AB\"");
field.text = "ab";
show("text = \"ab\"");

trace("// htmlText and setNewTextFormat, then same text");
fresh();
field.html = true;
field.htmlText = "<font size=\"30\">AB</font>";
show("htmlText = \"<font size='30'>AB</font>\"");
field.setNewTextFormat(mid);
show("setNewTextFormat(mid)");
field.text = "AB";
show("text = \"AB\"");

trace("// setNewTextFormat, setTextFormat, then same text");
fresh();
field.text = "AB";
show("text = \"AB\"");
field.setNewTextFormat(mid);
show("setNewTextFormat(mid)");
field.setTextFormat(big);
show("setTextFormat(big)");
field.text = "AB";
show("text = \"AB\"");

trace("// setNewTextFormat, htmlText, then same text");
fresh();
field.html = true;
field.text = "AB";
show("text = \"AB\"");
field.setNewTextFormat(mid);
show("setNewTextFormat(mid)");
field.htmlText = "AB";
show("htmlText = \"AB\"");
field.setTextFormat(big);
show("setTextFormat(big)");
field.text = "AB";
show("text = \"AB\"");

trace("// setNewTextFormat, different text, then same text");
fresh();
field.text = "AB";
show("text = \"AB\"");
field.setNewTextFormat(mid);
show("setNewTextFormat(mid)");
field.text = "XY";
show("text = \"XY\"");
field.setTextFormat(big);
show("setTextFormat(big)");
field.text = "XY";
show("text = \"XY\"");

trace("// setNewTextFormat, textColor, then same text");
fresh();
field.text = "AB";
show("text = \"AB\"");
field.setTextFormat(big);
show("setTextFormat(big)");
field.setNewTextFormat(mid);
show("setNewTextFormat(mid)");
field.textColor = 0xFF0000;
show("textColor = 0xFF0000");
field.text = "AB";
show("text = \"AB\"");

trace("// textColor, setNewTextFormat, then same text");
fresh();
field.text = "AB";
show("text = \"AB\"");
field.setTextFormat(big);
show("setTextFormat(big)");
field.textColor = 0xFF0000;
show("textColor = 0xFF0000");
field.setNewTextFormat(mid);
show("setNewTextFormat(mid)");
field.text = "AB";
show("text = \"AB\"");

function prepareHtml() {
    _root.createTextField("field" + depth, depth, 0, 0, 200, 40);
    field = _root["field" + depth];
    depth++;
    field.html = true;
    field.multiline = true;
    field.htmlText = "<p align=\"center\">x</p>";
    field.replaceText(1, 2, "");
    showHtml("prepared");
}

function showHtml(label) {
    trace("  " + label + ": length=" + field.length + " size=" + field.getTextFormat(0).size
        + " new=" + field.getNewTextFormat().size);
}

trace("// same htmlText");
prepareHtml();
field.htmlText = field.htmlText;
showHtml("htmlText = field.htmlText");

trace("// setNewTextFormat (own value), same htmlText");
prepareHtml();
field.setNewTextFormat(field.getNewTextFormat());
showHtml("setNewTextFormat(field.getNewTextFormat())");
field.htmlText = field.htmlText;
showHtml("htmlText = field.htmlText");

trace("// setNewTextFormat, same htmlText twice");
prepareHtml();
field.setNewTextFormat(field.getNewTextFormat());
showHtml("setNewTextFormat(field.getNewTextFormat())");
field.htmlText = field.htmlText;
showHtml("htmlText = field.htmlText");
field.replaceText(1, 2, "");
showHtml("replaceText(1, 2, \"\")");
field.htmlText = field.htmlText;
showHtml("htmlText = field.htmlText");
