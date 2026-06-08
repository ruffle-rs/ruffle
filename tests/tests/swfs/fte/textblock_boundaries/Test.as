package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var tb:TextBlock = new TextBlock(new TextElement("one  two\tthree", new ElementFormat(fd, 16)));

        trace("next atom 0: " + tb.findNextAtomBoundary(0));
        trace("next atom end clamps: " + tb.findNextAtomBoundary(99));
        trace("prev atom 0 clamps: " + tb.findPreviousAtomBoundary(0));
        trace("prev atom 5: " + tb.findPreviousAtomBoundary(5));
        trace("next word 0: " + tb.findNextWordBoundary(0));
        trace("next word 4: " + tb.findNextWordBoundary(4));
        trace("previous word 8: " + tb.findPreviousWordBoundary(8));
        trace("previous word 14: " + tb.findPreviousWordBoundary(14));
    }
}
}
