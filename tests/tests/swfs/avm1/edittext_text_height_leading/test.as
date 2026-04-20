function setUpField(field, size, leading, multiline) {
    field.border = true;
    field.embedFonts = true;
    field.multiline = multiline;

    var tf = field.getNewTextFormat();
    tf.size = size;
    tf.font = "TestFont";
    tf.leading = leading;
    field.setNewTextFormat(tf);

    if (multiline) {
        field.text = "acbd\nabcd";
    } else {
        field.text = "acbd";
    }
}

function printProps(field) {
    trace("Field: " + field);
    trace("  textHeight: " + field.textHeight);
}

Stage.scaleMode = "noScale";

var text1 = _root.createTextField(
    "text1",
    _root.getNextHighestDepth(),
    10, 10, 100, 30);
setUpField(text1, 20, 0, false);
printProps(text1);

var text2 = _root.createTextField(
    "text2",
    _root.getNextHighestDepth(),
    10, 50, 100, 40);
setUpField(text2, 30, 0, false);
printProps(text2);

var text3 = _root.createTextField(
    "text3",
    _root.getNextHighestDepth(),
    10, 100, 100, 40);
setUpField(text3, 30, 5, false);
printProps(text3);

var text4 = _root.createTextField(
    "text4",
    _root.getNextHighestDepth(),
    10, 150, 100, 60);
setUpField(text4, 20, 0, true);
printProps(text4);

var text5 = _root.createTextField(
    "text5",
    _root.getNextHighestDepth(),
    10, 220, 100, 80);
setUpField(text5, 30, 0, true);
printProps(text5);

var text6 = _root.createTextField(
    "text6",
    _root.getNextHighestDepth(),
    10, 310, 100, 80);
setUpField(text6, 30, 5, true);
printProps(text6);

var text7 = _root.createTextField(
    "text7",
    _root.getNextHighestDepth(),
    120, 10, 100, 120);
setUpField(text7, 30, 0, true);
text7.text = "acbd\nabcd\nacbd";
printProps(text7);

var text8 = _root.createTextField(
    "text8",
    _root.getNextHighestDepth(),
    120, 140, 100, 120);
setUpField(text8, 30, 5, true);
text8.text = "acbd\nabcd\nacbd";
printProps(text8);

var text9 = _root.createTextField(
    "text9",
    _root.getNextHighestDepth(),
    230, 10, 100, 120);
setUpField(text9, 30, 0, true);
text9.wordWrap = true;
text9.text = "acbd abcd\nacbd";
printProps(text9);

var text10 = _root.createTextField(
    "text10",
    _root.getNextHighestDepth(),
    230, 140, 100, 120);
setUpField(text10, 30, 5, true);
text10.wordWrap = true;
text10.text = "acbd abcd\nacbd";
printProps(text10);
