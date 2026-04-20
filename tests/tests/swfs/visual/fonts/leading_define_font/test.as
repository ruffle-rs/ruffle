var nextX:int = 0;
var nextY:int = 0;
var i:int = 0;

function newTextField(font, leading) {
    var text = _root.createTextField(
        "text" + (i++),
        _root.getNextHighestDepth(),
        nextX, nextY, 50, 80);

    text.border = true;
    text.embedFonts = true;
    var tf:TextFormat = new TextFormat();
    tf.font = font;
    tf.size = 20;
    if (leading != null) {
        tf.leading = leading;
    }
    text.setNewTextFormat(tf);

    text.multiline = true;
    text.text = "acbd\nabcd";

    nextX += text._width + 2;
    return text;
}

Stage.scaleMode = "noScale";

newTextField("TestFont2Gap0", null);
newTextField("TestFont2Gap0", 0);
newTextField("TestFont2Gap0", 2);
nextY += 100;
nextX = 0;
newTextField("TestFont2Gap100", null);
newTextField("TestFont2Gap100", 0);
newTextField("TestFont2Gap100", 2);
nextY += 100;
nextX = 0;

newTextField("TestFont3Gap0");
newTextField("TestFont3Gap0", 0);
newTextField("TestFont3Gap0", 2);
nextY += 100;
nextX = 0;
newTextField("TestFont3Gap100");
newTextField("TestFont3Gap100", 0);
newTextField("TestFont3Gap100", 2);
nextY += 100;
nextX = 0;
