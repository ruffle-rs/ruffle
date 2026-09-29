var tf = _root.createTextField("tf", 100, 0, 0, 400, 300);
tf.html = true;
tf.multiline = true;

var format = new TextFormat();
format.font = "Times";
format.size = 12;
tf.setNewTextFormat(format);

function runCase(label, html) {
    tf.htmlText = html;

    trace("CASE=" + label);
    trace("TEXT=" + tf.text);
    trace("HTML=" + tf.htmlText);
    trace("---");
}

runCase("basic",
    "A<img src='missing.jpg'>B"
);

runCase("attrs",
    "A<img src='missing.jpg' id='hello' width='40.9' height='30.7' align='right' hspace='4.9' vspace='6.2'>B"
);

runCase("left",
    "A<img src='missing.jpg' align='left'>B"
);

runCase("unsupported-align",
    "A<img src='missing.jpg' align='middle'>B"
);

runCase("no-src",
    "A<img id='hello' width='20' height='30'>B"
);

runCase("empty-src",
    "A<img src=''>B"
);

runCase("escaping",
    "A<img src='foo&amp;bar.jpg' id='a&amp;b'>B"
);

runCase("invalid-size",
    "A<img src='missing.jpg' width='abc' height='xyz'>B"
);

runCase("multiple",
    "A<img src='one.jpg'>B<img src='two.jpg'>C"
);

stop();
