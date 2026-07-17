package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    function Test() {
        var tb:TextBlock = new TextBlock();
        tb.content = new TextElement("test", new ElementFormat());
        var tl:TextLine = tb.createTextLine(null, 100);
        trace(tl.name);
    }
}
}
