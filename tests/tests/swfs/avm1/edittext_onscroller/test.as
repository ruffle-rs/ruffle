Stage.scaleMode = "noScale";

var tf = createTextField("tf", 3, 0, 0, 100, 100);
tf.embedFonts = true;
tf.setNewTextFormat(new TextFormat("SpaceAB", 20));
tf.multiline = true;
tf.border = true;
tf.type = "input";
tf.text = "";
for (var i = 0; i < 20; ++i) {
    tf.text += "abababababababababababababababababababab\n";
}

tf.onScroller = function(x) {
    trace("onScroller1: " + x);
    trace("  tf.scroll=" + tf.scroll);
};

tf.onScroller = function(x) {
    trace("onScroller2: " + x);
    trace("  tf.scroll=" + tf.scroll);
};

tf.scroll = 2;
tf.scroll = 1;
