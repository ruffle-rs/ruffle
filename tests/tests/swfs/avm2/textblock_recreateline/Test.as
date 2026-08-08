package {
    import flash.display.MovieClip;
    import flash.events.Event;
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

            // Test that properties of a TextLine get reset after calling recreateTextLine

            addChild(line2);

            line2.blendMode = "hardlight";
            line2.userData = "hello world!";
            line2.alpha = 0.5;
            line2.mask = this;
            line2.scaleX = 0.5;
            line2.scaleY = 0.5;
            line2.scaleZ = 0.5;
            line2.rotation = 45;
            line2.rotationX = 45;
            line2.rotationY = 45;
            line2.rotationZ = 45;
            line2.visible = false;
            line2.cacheAsBitmap = true;
            line2.opaqueBackground = 0xFF0000;
            line2.z = 30;

            trace(line2.name.indexOf("instance"));
            line2.name = "line#2";

            line2.addEventListener("customEvent", function(e) {
                trace("Event listener called");
            });
            line2.dispatchEvent(new Event("customEvent"));

            line2.addChild(new MovieClip());
            trace(line2.numChildren);

            block.recreateTextLine(line2, line1, 1200);

            trace(line2.textBlockBeginIndex);
            trace(line2.specifiedWidth);
            trace(line2.atomCount);
            trace(line2.rawTextLength);
            trace(line2.ascent);
            trace(line2.descent);

            trace(line2.blendMode);
            trace(line2.userData);
            trace(line2.alpha);
            trace(line2.mask);
            trace(line2.scaleX);
            trace(line2.scaleY);
            trace(line2.scaleZ);
            trace(line2.rotation);
            trace(line2.rotationX);
            trace(line2.rotationY);
            trace(line2.rotationZ);
            trace(line2.visible);
            trace(line2.cacheAsBitmap);
            trace(line2.opaqueBackground);
            trace(line2.z);

            trace(line2.name);

            trace(line2.parent);
            trace(line2.stage);
            trace(line2.x);
            trace(line2.y);
            line2.dispatchEvent(new Event("customEvent"));

            trace(line2.numChildren);

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

            var content2:ContentElement = new TextElement("Orem\nlay\nipsum");
            content2.elementFormat = new ElementFormat();
            var block2:TextBlock = new TextBlock(content2);

            var block2Line0:TextLine = block2.createTextLine(null, 1000);
            var block2Line1:TextLine = block2.createTextLine(block2Line0, 1000);
            var block2Line2:TextLine = block2.createTextLine(block2Line1, 1000);

            linesList.push(block2Line0);
            linesList.push(block2Line1);
            linesList.push(block2Line2);

            // What happens if the original line belongs to a different block?
            try {
                block.recreateTextLine(block2Line1, line1, 1000);
            } catch(e:Error) {
                trace("Error: " + e.getStackTrace());
            }

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

            // What happens if the original line is at the start of the current line?
            block.recreateTextLine(line0, line3, 1000);

            dumpInfo(block, linesList);

            // Error cases
            try {
                block.recreateTextLine(null, line1);
            } catch(e:Error) {
                trace("Error: " + e.getStackTrace());
            }

            try {
                line3.validity = "invalid";
                block.recreateTextLine(line4, line3);
            } catch(e:Error) {
                trace("Error: " + e.getStackTrace());
            }

            try {
                block.recreateTextLine(null, line1, -1);
            } catch(e:Error) {
                trace("Error: " + e.getStackTrace());
            }

            try {
                block.recreateTextLine(line3, line1, -1);
            } catch(e:Error) {
                trace("Error: " + e.getStackTrace());
            }
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
