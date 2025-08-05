
var text = _root.createTextField(
    "text",
    _root.getNextHighestDepth(),
    0, 0, 380, 50);

text.border = true;
text.embedFonts = true;
var tf:TextFormat = new TextFormat();
tf.font = "TestFont";
tf.size = 10;
text.setNewTextFormat(tf);

text.multiline = true;
text.text = "aa\nbab";

Stage.scaleMode = "noScale";
