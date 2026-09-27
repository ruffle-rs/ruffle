Stage.scaleMode = "noScale";
Stage.align = "TL";

var nextDepth:Number = 100;
var nextY:Number = 10;

function newField(name:String):TextField {
    _root.createTextField(name, nextDepth++, 10, nextY, 520, 32);

    var tf:TextField = _root[name];
    tf.html = true;
    tf.multiline = true;
    tf.wordWrap = true;
    tf.border = true;

    var fmt:TextFormat = new TextFormat();
    fmt.font = "Arial";
    fmt.size = 12;
    tf.setNewTextFormat(fmt);

    nextY += 38;
    return tf;
}

function report(name:String, tf:TextField):Void {
    trace("CASE=" + name);
    trace("TEXT=[" + tf.text + "]");
    trace("LENGTH=" + tf.text.length);
    trace("TEXTWIDTH=" + tf.textWidth);
    trace("HTML=[" + tf.htmlText + "]");
    trace("---");
}

function setHtml(tf:TextField, body:String):Void {
    tf.htmlText =
        '<FONT FACE="Arial" SIZE="12">' +
        body +
        '</FONT>';
}

var tf:TextField;

tf = newField("case01");
setHtml(tf, 'A<img src="test_image.jpg" width="40" height="24" hspace="0" vspace="0">B');
report("01_BASIC", tf);

tf = newField("case02");
setHtml(tf, 'A<img src="test_image.jpg">B');
report("02_INTRINSIC", tf);

tf = newField("case03");
setHtml(tf, 'A<img src="test_image.jpg" width="40">B');
report("03_WIDTH_ONLY", tf);

tf = newField("case04");
setHtml(tf, 'A<img src="test_image.jpg" height="24">B');
report("04_HEIGHT_ONLY", tf);

tf = newField("case05");
setHtml(tf, 'A<img src="test_image.jpg" width="0" height="0">B');
report("05_ZERO_SIZE", tf);

tf = newField("case06");
setHtml(tf, 'A<img src="test_image.jpg" width="-40" height="-24" hspace="0" vspace="0">B');
report("06_NEGATIVE_SIZE", tf);

tf = newField("case07");
setHtml(tf, 'A<img src="test_image.jpg" width="40.5" height="20.25" hspace="0" vspace="0">B');
report("07_DECIMAL_SIZE", tf);

tf = newField("case08");
setHtml(tf, 'A<img src="test_image.jpg" width="invalid" height="invalid">B');
report("08_INVALID_SIZE", tf);

tf = newField("case09");
setHtml(tf, 'A<img width="40" height="24">B');
report("09_NO_SRC", tf);

tf = newField("case10");
setHtml(tf, 'A<img src="" width="40" height="24">B');
report("10_EMPTY_SRC", tf);
