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
    tf.background = true;
    tf.backgroundColor = 0xCCFFFF;

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
    trace("---");
}

function setHtml(tf:TextField, body:String):Void {
    tf.htmlText =
        '<FONT FACE="Arial" SIZE="12">' +
        body +
        '</FONT>';
}

function label(text:String, x:Number, y:Number):Void {
    _root.createTextField("label" + nextDepth, nextDepth++, x, y, 125, 16);

    var tf:TextField = _root["label" + (nextDepth - 1)];
    tf.text = text;

    var fmt:TextFormat = new TextFormat();
    fmt.font = "Arial";
    fmt.size = 10;
    tf.setTextFormat(fmt);
}

var tf:TextField;
var words:String =
    "ONE TWO THREE FOUR FIVE SIX SEVEN EIGHT NINE TEN " +
    "ELEVEN TWELVE THIRTEEN FOURTEEN FIFTEEN SIXTEEN.";

label("01 LEFT", 5, 4);
tf = makeField("case01", 5, 20, 260, 54);
setHtml(tf,
    '<img src="test_image.jpg" width="40" height="40" align="left" hspace="0" vspace="0">' +
    words
);
report("01_ALIGN_LEFT", tf);

label("02 RIGHT", 280, 4);
tf = makeField("case02", 280, 20, 260, 54);
setHtml(tf,
    '<img src="test_image.jpg" width="40" height="40" align="right" hspace="0" vspace="0">' +
    words
);
report("02_ALIGN_RIGHT", tf);

label("03 DEFAULT", 5, 82);
tf = makeField("case03", 5, 98, 260, 54);
setHtml(tf,
    '<img src="test_image.jpg" width="40" height="40" hspace="0" vspace="0">' +
    words
);
report("03_NO_ALIGN", tf);

label("04 H/V SPACE", 280, 82);
tf = makeField("case04", 280, 98, 260, 68);
setHtml(tf,
    '<img src="test_image.jpg" width="40" height="40" align="left" hspace="8" vspace="8">' +
    words
);
report("04_SPACE_EIGHT", tf);

label("05 BEFORE LEFT", 5, 174);
tf = makeField("case05", 5, 190, 260, 68);
setHtml(tf,
    'PREFIX PREFIX ' +
    '<img src="test_image.jpg" width="40" height="40" align="left" hspace="0" vspace="0">' +
    words
);
report("05_TEXT_BEFORE_LEFT", tf);

label("06 BEFORE RIGHT", 280, 174);
tf = makeField("case06", 280, 190, 260, 68);
setHtml(tf,
    'PREFIX PREFIX ' +
    '<img src="test_image.jpg" width="40" height="40" align="right" hspace="0" vspace="0">' +
    words
);
report("06_TEXT_BEFORE_RIGHT", tf);

label("07 MULTIPLE", 5, 266);
tf = makeField("case07", 5, 282, 260, 62);
setHtml(tf,
    'A' +
    '<img src="test_image.jpg" width="30" height="30" hspace="0" vspace="0">' +
    'B' +
    '<img src="test_image.jpg" width="24" height="24" hspace="0" vspace="0">' +
    'C'
);
report("07_MULTIPLE", tf);

label("08 OTHER ALIGN", 280, 266);
tf = makeField("case08", 280, 282, 260, 62);
setHtml(tf,
    'A<img src="test_image.jpg" width="40" height="40" align="middle" hspace="0" vspace="0">B'
);
report("08_ALIGN_OTHER", tf);
