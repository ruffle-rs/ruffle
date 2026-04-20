text.border = true;
text._x = 50;
text._width = 100;

var o = new Object(); o.toString = function() { return 'center'; };

var values = [
    true, false, "true", "false",
    "text", 0, -1, 1, 2, 0.4, 2.4, -3.6,
    "lEft", "RIght", "rightf",
    "none", null, undefined, function(){},
    o
];

var i = 0;
while (i < values.length) {
    text.autoSize = values[i];
    trace(text.autoSize);
    i += 1;
}
