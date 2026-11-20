import flash.display.BitmapData;

function hex6(v) {
    var s = v.toString(16);
    while (s.length < 6) {
        s = "0" + s;
    }
    return "0x" + s;
}

Stage.scaleMode = "noScale";

var tf = _root.createTextField("tf", 1, 0, 0, 100, 50);
tf.embedFonts = false;
tf.multiline = true;
tf.wordWrap = true;
tf.background = true;
tf.backgroundColor = 0xFFFFFF;
tf.textColor = 0x000000;

var fmt = new TextFormat();
fmt.font = "TestFont";
fmt.size = 20;
tf.setNewTextFormat(fmt);
tf.text = "bbbb\nbbbb\nbbbb";

trace("initial scroll=" + tf.scroll + " bottomScroll=" + tf.bottomScroll);

var bmp1 = new BitmapData(100, 50, false, 0x00FF00);
bmp1.draw(tf);
trace("scroll=1 line1(10,8)=" + hex6(bmp1.getPixel(10, 8)));
trace("scroll=1 line2(10,28)=" + hex6(bmp1.getPixel(10, 28)));
trace("scroll=1 culled_line3(10,45)=" + hex6(bmp1.getPixel(10, 45)));

tf.scroll = 2;
trace("scrolled scroll=" + tf.scroll + " bottomScroll=" + tf.bottomScroll);

var bmp2 = new BitmapData(100, 50, false, 0x00FF00);
bmp2.draw(tf);
trace("scroll=2 line2(10,8)=" + hex6(bmp2.getPixel(10, 8)));
trace("scroll=2 line3(10,28)=" + hex6(bmp2.getPixel(10, 28)));

// Tiny field still renders the first visible line even when taller than bounds.
var tiny = _root.createTextField("tiny", 2, 0, 0, 100, 10);
tiny.embedFonts = false;
tiny.multiline = true;
tiny.wordWrap = true;
tiny.background = true;
tiny.backgroundColor = 0xFFFFFF;
tiny.textColor = 0x000000;
tiny.setNewTextFormat(fmt);
tiny.text = "bbbb\nbbbb";
trace("tiny scroll=" + tiny.scroll + " bottomScroll=" + tiny.bottomScroll);

var bmp3 = new BitmapData(100, 10, false, 0x00FF00);
bmp3.draw(tiny);
trace("tiny line1(10,5)=" + hex6(bmp3.getPixel(10, 5)));
