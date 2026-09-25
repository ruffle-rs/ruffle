package {
import flash.display.*;
import flash.text.engine.*;

public class Test extends Sprite {
    public function Test() {
        var tl:TextLine = getTextLine();
        testCall(function() { tl.contextMenu = null; });
        testCall(function() { tl.focusRect = null; });
        testCall(function() { tl.tabChildren = null; });
        testCall(function() { tl.tabEnabled = null; });
        testCall(function() { tl.tabIndex = null; });
    }

    private function testCall(f:Function):void {
        try {
            f();
            trace("Did not throw");
        } catch (e) {
            trace("Threw: " + e.getStackTrace());
        }
    }

    private function getTextLine():TextLine {
        var block:TextBlock = new TextBlock(new TextElement("a", getElementFormat()));
        return block.createTextLine(null, 10000);
    }

    private function getElementFormat():ElementFormat {
        var fd:FontDescription = new FontDescription();
        return new ElementFormat(fd, 20);
    }
}
}
