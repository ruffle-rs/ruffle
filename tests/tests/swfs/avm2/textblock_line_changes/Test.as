package {
    import flash.display.MovieClip;
    import flash.text.engine.*;

    public class Test extends MovieClip {
        public function Test() {
            var content:ContentElement = new TextElement("Lorem ipsum dolor\nsit amet, consectetur adipiscing elit, sed\ndo eiusmod tempor\nincididunt ut labore\net dolore magna\naliqua. Ut enim ad");
            content.elementFormat = new ElementFormat();
            var block:TextBlock = new TextBlock(content);

            var line0:TextLine = block.createTextLine(null, 1000);
            line0.y = 20;
            addChild(line0);

            var line1:TextLine = block.createTextLine(line0, 1000);
            line1.y = 40;
            addChild(line1);

            var line2:TextLine = block.createTextLine(line1, 1000);
            line2.y = 60;
            addChild(line2);

            var line3:TextLine = block.createTextLine(line2, 1000);
            line3.y = 80;
            addChild(line3);

            var line4:TextLine = block.createTextLine(line3, 1000);
            line4.y = 100;
            addChild(line4);

            var linesList:Array = [line0, line1, line2, line3, line4];

            dumpInfo(block, linesList);

            line2.validity = "invalid";

            dumpInfo(block, linesList);

            line2.validity = "static";

            dumpInfo(block, linesList);

            var recreateResult:TextLine = block.recreateTextLine(line1, line0, 1000);
            
            trace("Calling recreateTextLine returns the same line: " + (line1 === recreateResult));

            dumpInfo(block, linesList);

            // At this point, `block` is actually maintaining *two* double-linked
            // lists of lines- line0, line1, and line4 are in one linked list,
            // while line2 and line3 are in another linked list. Let's see what
            // happens if we try to releaseLines(line1, line2). They're in the
            // same text block, but not in the same linked list.

            trace("Calling releaseLines(line1, line2)");
            try {
                block.releaseLines(line1, line2);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("Calling releaseLines(line2, line3)");
            try {
                block.releaseLines(line2, line3);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            line3.validity = "static";

            dumpInfo(block, linesList);

            var createResult:TextLine = block.createTextLine(null, 1000);

            trace("Calling createTextLine returns the same line: " + (line0 === createResult));

            dumpInfo(block, linesList);

            line4.validity = "static";

            dumpInfo(block, linesList);

            // Reset state
            block.recreateTextLine(line0, null, 1000);
            line0.y = 20;

            block.recreateTextLine(line1, line0, 1000);
            line1.y = 40;

            block.recreateTextLine(line2, line1, 1000);
            line2.y = 60;

            block.recreateTextLine(line3, line2, 1000);
            line3.y = 80;

            block.recreateTextLine(line4, line3, 1000);
            line4.y = 100;

            dumpInfo(block, linesList);

            trace("Take two!");

            // Once again, cause the TextBlock to contain two separate
            // doubly-linked lists.
            var recreateResult:TextLine = block.recreateTextLine(line1, line0, 1000);

            dumpInfo(block, linesList);

            trace("Calling releaseLines(line1, line2)");
            try {
                block.releaseLines(line1, line2);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("Calling releaseLines(line2, line3)");
            try {
                block.releaseLines(line2, line3);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            // Reset state
            block.recreateTextLine(line0, null, 1000);
            line0.y = 20;

            block.recreateTextLine(line1, line0, 1000);
            line1.y = 40;

            block.recreateTextLine(line2, line1, 1000);
            line2.y = 60;

            block.recreateTextLine(line3, line2, 1000);
            line3.y = 80;

            block.recreateTextLine(line4, line3, 1000);
            line4.y = 100;

            dumpInfo(block, linesList);

            var line5:TextLine = block.createTextLine(line4, 1000);
            line5.y = 120;
            addChild(line5);

            linesList.push(line5);

            dumpInfo(block, linesList);

            block.createTextLine(line4, 1000);

            dumpInfo(block, linesList);

            // Reset state
            block.recreateTextLine(line0, null, 1000);
            line0.y = 20;

            block.recreateTextLine(line1, line0, 1000);
            line1.y = 40;

            block.recreateTextLine(line2, line1, 1000);
            line2.y = 60;

            block.recreateTextLine(line3, line2, 1000);
            line3.y = 80;

            block.recreateTextLine(line4, line3, 1000);
            line4.y = 100;

            block.recreateTextLine(line5, line4, 1000);
            line5.y = 120;

            block.releaseLines(line2, line4);

            dumpInfo(block, linesList);

            block.releaseLines(line5, line5);

            dumpInfo(block, linesList);

            // Reset state
            block.recreateTextLine(line0, null, 1000);
            line0.y = 20;

            block.recreateTextLine(line1, line0, 1000);
            line1.y = 40;

            block.recreateTextLine(line2, line1, 1000);
            line2.y = 60;

            block.recreateTextLine(line3, line2, 1000);
            line3.y = 80;

            block.recreateTextLine(line4, line3, 1000);
            line4.y = 100;

            block.recreateTextLine(line5, line4, 1000);
            line5.y = 120;

            // Set content back to itself, see if that does anything
            block.content = block.content;

            dumpInfo(block, linesList);

            // Reset state
            block.recreateTextLine(line0, null, 1000);
            line0.y = 20;

            block.recreateTextLine(line1, line0, 1000);
            line1.y = 40;

            block.recreateTextLine(line2, line1, 1000);
            line2.y = 60;

            block.recreateTextLine(line3, line2, 1000);
            line3.y = 80;

            block.recreateTextLine(line4, line3, 1000);
            line4.y = 100;

            block.recreateTextLine(line5, line4, 1000);
            line5.y = 120;

            var newElement:TextElement = new TextElement("some text here");
            newElement.elementFormat = new ElementFormat();
            block.content = newElement;

            dumpInfo(block, linesList);
        }

        static function dumpInfo(block:TextBlock, linesList:Array):void {
            trace("First line in block: " + formatTextLine(block.firstLine, linesList));
            trace("Last line in block: " + formatTextLine(block.lastLine, linesList));
            for (var i:int = 0; i < linesList.length; i ++) {
                var line:TextLine = linesList[i];
                trace("Line #" + i + ":");
                trace("    line.validity: " + line.validity);
                trace("    line.textBlock: " + line.textBlock);
                trace("    line.previousLine: " + formatTextLine(line.previousLine, linesList));
                trace("    line.nextLine: " + formatTextLine(line.nextLine, linesList));
            }
        }

        static function formatTextLine(line:TextLine, linesList:Array):String {
            if (line === null) {
                return "null";
            } else {
                var index = linesList.indexOf(line);
                if (index > -1) {
                    return "line-" + index;
                } else {
                    return "line-unknown";
                }
            }
        }
    }
}
