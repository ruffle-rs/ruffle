package {
import flash.display.Sprite;

public class Test extends Sprite {
    function Test() {

    }
}
}

import flash.text.engine.ElementFormat;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

var ef:ElementFormat = new ElementFormat();

trace("=== Single line ===");
var tb1:TextBlock = new TextBlock(new TextElement("Hello", ef));
var line1:TextLine = tb1.createTextLine(null, 500);
trace("line1 != null: " + (line1 != null));
trace("rawTextLength: " + line1.rawTextLength);
trace("textBlockBeginIndex: " + line1.textBlockBeginIndex);
trace("result: " + tb1.textLineCreationResult);

// Next call should return null (text exhausted).
var line1b:TextLine = tb1.createTextLine(line1, 500);
trace("line1b == null: " + (line1b == null));
trace("result after complete: " + tb1.textLineCreationResult);

trace("=== Null content ===");
var tb2:TextBlock = new TextBlock();
var nullLine:TextLine = tb2.createTextLine(null, 500);
trace("null content line: " + nullLine);

// recreateTextLine returns same object
trace("=== recreateTextLine ===");
var tb3:TextBlock = new TextBlock(new TextElement("ABCDEF", ef));
var rl:TextLine = tb3.createTextLine(null, 500);
trace("before recreate rawTextLength: " + rl.rawTextLength);

var rl2:TextLine = tb3.recreateTextLine(rl, null, 500);
trace("same object: " + (rl === rl2));
trace("after recreate rawTextLength: " + rl2.rawTextLength);
trace("after recreate beginIndex: " + rl2.textBlockBeginIndex);
trace("result: " + tb3.textLineCreationResult);

trace("=== firstLine ===");
var tb4:TextBlock = new TextBlock(new TextElement("Test", ef));
trace("firstLine before: " + tb4.firstLine);
var fl:TextLine = tb4.createTextLine(null, 500);
trace("firstLine after: " + (tb4.firstLine === fl));
