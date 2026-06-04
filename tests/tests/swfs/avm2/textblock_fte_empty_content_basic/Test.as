package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.FontLookup;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

[SWF(width="200", height="100")]
public class Test extends Sprite {
    [Embed(source="LiberationSans-Regular.ttf", fontName="Liberation Sans", embedAsCFF="true", unicodeRange="U+000A,U+000D,U+0020-U+007E")]
    private var LiberationSans:Class;

    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        fd.fontLookup = FontLookup.EMBEDDED_CFF;
        var fmt:ElementFormat = new ElementFormat(fd, 20);

        traceCase("empty", "", fmt);
        traceCase("space", " ", fmt);
        traceCase("twoSpaces", "  ", fmt);
        traceCase("newline", "\n", fmt);
    }

    private function traceCase(name:String, text:String, fmt:ElementFormat):void {
        var block:TextBlock = new TextBlock(new TextElement(text, fmt));
        var line:TextLine = block.createTextLine(null, 100);

        trace(name + ".line is null: " + (line === null));
        trace(name + ".creation result: " + block.textLineCreationResult);
        if (line !== null) {
            trace(name + ".rawTextLength: " + line.rawTextLength);
        }
    }
}
}
