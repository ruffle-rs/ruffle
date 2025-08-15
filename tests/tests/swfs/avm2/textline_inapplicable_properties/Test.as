package {
import flash.display.Sprite;

public class Test extends Sprite {
    function Test() {

    }
}
}

import flash.ui.ContextMenu;
import flash.text.engine.ElementFormat;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

function logError(f:*):void {
    try {
        f();
    } catch(error) {
        trace("Error: " + error);
    }
}

var tb:TextBlock = new TextBlock();
tb.content = new TextElement("test", new ElementFormat());
var tl:TextLine = tb.createTextLine(null, 100);

logError(function() {
    trace(tl.contextMenu = new ContextMenu());
});
trace(tl.contextMenu);

logError(function() {
    trace(tl.focusRect = true);
});
trace(tl.focusRect);

logError(function() {
    trace(tl.tabChildren = true);
});
trace(tl.tabChildren);

logError(function() {
    trace(tl.tabEnabled = true);
});
trace(tl.tabEnabled);

logError(function() {
    trace(tl.tabIndex = 3);
});
trace(tl.tabIndex);
