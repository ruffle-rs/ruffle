package {
    import flash.display.MovieClip;
    import flash.text.engine.*;

    public class Test extends MovieClip {
        public function Test() {
            var content:ContentElement = new TextElement("Lorem ipsum dolor\nsit amet, consectetur adipiscing elit, sed\ndo eiusmod tempor\nincididunt ut labore\net dolore magna");
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

            line3.validity = "static";

            dumpInfo(block, linesList);

            var createResult:TextLine = block.createTextLine(line0, null, 1000);

            trace("Calling createTextLine returns the same line: " + (line0 === createResult));

            dumpInfo(block, linesList);
        }

        static function dumpInfo(block:TextBlock, linesList:Array):void {
            trace("First line in block: " + formatTextLine(block.firstLine, linesList));
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
