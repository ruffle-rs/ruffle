// Entity references must be processed together with the surrounding text.
var cases = [
    ["space between entities", "&lt; &gt;"],
    ["newline entity", "x&#10;y"],
    ["entity in text", "Tom &amp; Jerry"],
    ["adjacent entities", "&amp;&amp;"],
    ["whitespace-only text between tags", "<b>&lt;</b> <b>&gt;</b>"],
    ["non-ASCII text around an entity", "café &amp; crème"]
];
for (var i = 0; i < cases.length; i++) {
    _root.createTextField("t" + i, i + 1, 0, 0, 200, 50);
    var tf = _root["t" + i];
    tf.html = true;
    tf.multiline = true;
    tf.htmlText = cases[i][1];
    var codes = [];
    for (var j = 0; j < tf.text.length; j++) {
        codes.push(tf.text.charCodeAt(j));
    }
    trace(cases[i][0] + ": " + codes.join(","));
}
