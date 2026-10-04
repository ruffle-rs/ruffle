function escapeText(value) {
    return value.split("\r").join("\\r").split("\n").join("\\n");
}

function newField() {
    var nextDepth = _root.getNextHighestDepth();
    var name = "tf" + nextDepth;
    _root.createTextField(name, nextDepth++, 0, 0, 400, 300);

    var tf = _root[name];
    tf.html = true;
    tf.multiline = true;

    var format = new TextFormat();
    format.font = "Times";
    format.size = 12;
    tf.setNewTextFormat(format);

    return tf;
}

function runCase(html) {
    var tf = newField();
    tf.htmlText = html;

    trace("sourceHtml=" + escapeText(html));
    trace("text=" + escapeText(tf.text));
    trace("resultHtml=" + tf.htmlText);
    trace("---");
}

function runStyleCase(css, html) {
    var tf = newField();

    var style = new TextField.StyleSheet();
    style.parseCSS(css);
    tf.styleSheet = style;

    tf.htmlText = html;

    trace("css=" + css);
    trace("sourceHtml=" + escapeText(html));
    trace("text=" + escapeText(tf.text));
    for (var i = 0; i < tf.length; i++) {
        var f = tf.getTextFormat(i, i + 1);
        trace("  [" + i + "] color=" + f.color + " bold=" + f.bold
            + " url=" + f.url + " align=" + f.align);
    }
    trace("---");
}

var cases = [
    // <font>
    "<font face='Arial' face='Verdana'>A</font>",
    "<font face='Arial' face='Verdana' face='Courier'>A</font>",
    "<font size='10' size='20'>A</font>",
    "<font size='10' size='abc'>A</font>",
    "<font size='10' size=''>A</font>",
    "<font size='abc' size='20'>A</font>",
    "<font size='10' size='+5'>A</font>",
    "<font color='#FF0000' color='#00FF00'>A</font>",
    "<font color='#FF0000' color='green'>A</font>",
    "<font color='#FF0000' color='#00FF00' color='#0000FF'>A</font>",
    "<font letterSpacing='1' letterSpacing='5'>A</font>",
    "<font letterSpacing='1' letterSpacing='abc'>A</font>",
    "<font kerning='1' kerning='0'>A</font>",
    "<font kerning='0' kerning='1'>A</font>",
    "<font kerning='1' kerning='2'>A</font>",
    "<font face='Arial' FACE='Verdana'>A</font>",
    "<font FACE='Arial' face='Verdana'>A</font>",
    "<font color='#FF0000' size='10' color='#00FF00' size='20'>A</font>",

    // <p>
    "<p align='left' align='right'>A</p>",
    "<p align='right' align='center'>A</p>",
    "<p align='center' align='justify'>A</p>",
    "<p align='right' align='something'>A</p>",
    "<p align='right' align=''>A</p>",
    "<p align='right' ALIGN='center'>A</p>",

    // <a>
    "<a href='http://a.example/' href='http://b.example/'>A</a>",
    "<a href='http://a.example/' href=''>A</a>",
    "<a target='_blank' target='_self' href='http://a.example/'>A</a>",
    "<a href='http://a.example/' target='_blank' href='http://b.example/' target='_self'>A</a>",
    "<a href='http://a.example/' HREF='http://b.example/'>A</a>",

    // <textformat>
    "<textformat leftmargin='1' leftmargin='5'>A</textformat>",
    "<textformat rightmargin='1' rightmargin='5'>A</textformat>",
    "<textformat indent='1' indent='5'>A</textformat>",
    "<textformat blockindent='1' blockindent='5'>A</textformat>",
    "<textformat leading='1' leading='5'>A</textformat>",
    "<textformat tabstops='10,20' tabstops='30,40,50'>A</textformat>",
    "<textformat leftmargin='5' leftmargin='abc'>A</textformat>",
    "<textformat tabstops='10,20' tabstops=''>A</textformat>",
    "<textformat indent='1' INDENT='5'>A</textformat>",

    // <img>
    "A<img src='one.jpg' src='two.jpg'>B",
    "A<img src='one.jpg' src=''>B",
    "A<img src='one.jpg' id='first' id='second'>B",
    "A<img src='one.jpg' width='10' width='20' height='30' height='40'>B",
    "A<img src='one.jpg' width='10' width='abc'>B",
    "A<img src='one.jpg' align='right' align='left'>B",
    "A<img src='one.jpg' align='left' align='right'>B",
    "A<img src='one.jpg' hspace='1' hspace='5' vspace='2' vspace='6'>B",
    "A<img src='one.jpg' checkPolicyFile='true' checkPolicyFile='false'>B",
    "A<img src='one.jpg' checkPolicyFile='false' checkPolicyFile='true'>B",
    "A<img src='one.jpg' SRC='two.jpg'>B",

    // Tags without meaningful attributes
    "<b x='1' x='2'>A</b>",
    "<li x='1' x='2'>A</li>",
    "A<br x='1' x='2'>B",

    // Nested tags with duplicates
    "<font color='#FF0000' color='#00FF00'><font size='10' size='20'>A</font>B</font>",
    "<p align='right' align='center'><font face='Arial' face='Verdana'>A</font></p>"
];

for (var i = 0; i < cases.length; i++) {
    runCase(cases[i]);
}

var css = ".red { color: #FF0000; } .blue { color: #0000FF; } .bold { font-weight: bold; }";
var styleCases = [
    "<p class='red' class='blue'>A</p>",
    "<span class='red' class='blue'>A</span>",
    "<a href='http://a.example/' class='red' class='blue'>A</a>",
    "<span class='red' CLASS='blue'>A</span>",
    "<span class='bold' class='red'>A</span>",
    "<span class='red' class='missing'>A</span>"
];

for (var i = 0; i < styleCases.length; i++) {
    runStyleCase(css, styleCases[i]);
}
