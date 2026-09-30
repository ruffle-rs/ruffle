var nextDepth = 100;

function escapeText(value) {
    return value.split("\r").join("\\r").split("\n").join("\\n");
}

function runCase(html, multiline, condenseWhite) {
    var name = "tf" + nextDepth;
    _root.createTextField(name, nextDepth++, 0, 0, 400, 300);

    var tf = _root[name];
    tf.html = true;
    tf.multiline = multiline;
    tf.condenseWhite = condenseWhite;

    if (tf.setNewTextFormat) {
        var format = new TextFormat();
        format.font = "Times";
        format.size = 12;
        tf.setNewTextFormat(format);
    }

    tf.htmlText = html;

    trace("SOURCE=" + escapeText(html));
    trace("MULTILINE=" + multiline);
    trace("CONDENSE_WHITE=" + condenseWhite);
    trace("TEXT=" + escapeText(tf.text));
    trace("HTML=" + tf.htmlText);
    trace("---");
}

function runReparseCase(html) {
    var name = "tf" + nextDepth;
    _root.createTextField(name, nextDepth++, 0, 0, 400, 300);

    var tf = _root[name];
    tf.html = true;
    tf.multiline = true;
    tf.condenseWhite = false;

    if (tf.setNewTextFormat) {
        var format = new TextFormat();
        format.font = "Times";
        format.size = 12;
        tf.setNewTextFormat(format);
    }

    tf.htmlText = html;

    var firstHtml = tf.htmlText;

    trace("REPARSE_SOURCE=[" + escapeText(html) + "]");
    trace("FIRST_TEXT=[" + escapeText(tf.text) + "]");
    trace("FIRST_HTML=[" + firstHtml + "]");

    tf.htmlText = firstHtml + " ";

    trace("SECOND_SOURCE=[" + escapeText(firstHtml + " ") + "]");
    trace("SECOND_TEXT=[" + escapeText(tf.text) + "]");
    trace("SECOND_HTML=[" + tf.htmlText + "]");
    trace("---");
}

function runDefaultCase(html) {
    runCase(html, true, false);
}

runDefaultCase(
    "A<img src='missing.jpg'>B"
);

runDefaultCase(
    "A<img src='missing.jpg' id='hello' width='40.9' height='30.7' align='right' hspace='4.9' vspace='6.2'>B"
);

runDefaultCase(
    "A<img src='missing.jpg' align='left'>B"
);

runDefaultCase(
    "A<img src='missing.jpg' align='middle'>B"
);

runDefaultCase(
    "A<img id='hello' width='20' height='30'>B"
);

runDefaultCase(
    "A<img>B"
);

runDefaultCase(
    "A<img src=''>B"
);

runDefaultCase(
    "A<img src='missing.jpg' />B"
);

runDefaultCase(
    "A<img src='foo&amp;bar.jpg' id='a&amp;b'>B"
);

runDefaultCase(
    "A<img src='missing.jpg' width='abc' height='xyz'>B"
);

runDefaultCase(
    "A<img src='one.jpg'>B<img src='two.jpg'>C"
);

runDefaultCase(
    "A   <img src='missing.jpg'>   B"
);

runCase(
    "A   <img src='missing.jpg'>   B",
    false,
    false
);

runCase(
    "A   <img src='missing.jpg'>   B",
    true,
    true
);

runCase(
    "A   <img src='missing.jpg'>   B",
    false,
    true
);

runCase(
    "A\n  <img src='missing.jpg'>\n  B",
    true,
    false
);

runCase(
    "A\n  <img src='missing.jpg'>\n  B",
    true,
    true
);

runDefaultCase(
    "A<img src='missing.jpg' align='LEFT'>B"
);

runDefaultCase(
    "A<img src='missing.jpg' align='Left'>B"
);

runDefaultCase(
    "A<img src='missing.jpg' align='something'>B"
);

runDefaultCase(
    "A<img src='missing.jpg' checkPolicyFile='true' unknown='value'>B"
);

runReparseCase(
    "A<img src='missing.jpg'>B"
);

runDefaultCase(
    "<font size='2'>A</font><img src='missing.jpg'>B"
);

runDefaultCase(
    "A<img src='missing.jpg'><font size='2'>B</font>"
);

runDefaultCase(
    "<font size='20'>A<img src='missing.jpg'>B</font>"
);

runDefaultCase(
    "<b>A<img src='missing.jpg'>B</b>"
);

runDefaultCase(
    "<i>A<img src='missing.jpg'>B</i>"
);

runDefaultCase(
    "<u>A<img src='missing.jpg'>B</u>"
);

runDefaultCase(
    "A<img src='missing.jpg'>X</img>B"
);

stop();
