Stage.scaleMode = "noScale";
Stage.align = "TL";

var nextDepth:Number = 100;

function makeField(
    name:String,
    x:Number,
    y:Number,
    width:Number,
    height:Number
):TextField {
    _root.createTextField(name, nextDepth++, x, y, width, height);

    var tf:TextField = _root[name];
    tf.html = true;
    tf.multiline = true;
    tf.wordWrap = true;
    tf.border = true;

    var fmt:TextFormat = new TextFormat();
    fmt.font = "Arial";
    fmt.size = 12;
    tf.setNewTextFormat(fmt);

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

tf = makeField("case01", 10, 10, 520, 70);
setHtml(
    tf,
    'MISSING: A<img src="does_not_exist.jpg" width="40" height="40">B'
);
report("01_MISSING_FILE", tf);

tf = makeField("case02", 10, 90, 520, 90);
setHtml(
    tf,
    'SWF: A<img src="child.swf" width="100" height="50" hspace="0" vspace="0">B'
);
report("02_SWF", tf);

tf = makeField("case03", 10, 190, 520, 90);
setHtml(
    tf,
    'FIRST: A<img src="test_image.jpg" width="120" height="120">B'
);
setHtml(
    tf,
    'SECOND: A<img src="test_image.jpg" width="40" height="40">B'
);
report("03_RACE", tf);
