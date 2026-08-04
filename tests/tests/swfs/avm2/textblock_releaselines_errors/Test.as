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

            var line5:TextLine = block.createTextLine(line4, 1000);
            line5.y = 120;
            addChild(line5);

            var linesList:Array = [line0, line1, line2, line3, line4, line5];

            trace("!!!! initialized list of six lines, line0..=line5");

            dumpInfo(block, linesList);

            trace("!! block.releaseLines(null, null);");
            try {
                block.releaseLines(null, null);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("!! block.releaseLines(null, line4);");
            try {
                block.releaseLines(null, line4);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("!! block.releaseLines(line1, null);");
            try {
                block.releaseLines(line1, null);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("!! line1.validity = \"static\";");
            line1.validity = "static";

            trace("!! block.releaseLines(line1, line2);");
            try {
                block.releaseLines(line1, line2);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            dumpInfo(block, linesList);

            trace("!! line3.validity = \"invalid\";");
            line3.validity = "invalid";

            trace("!! block.releaseLines(line3, line4);");
            try {
                block.releaseLines(line3, line4);
            } catch(e:Error) {
                trace(e.getStackTrace());
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

            block.recreateTextLine(line5, line4, 1000);
            line5.y = 120;

            trace("!!!! reset all lines");

            // This will invalidate `line5` and should dissociate it from the
            // `TextBlock`
            trace("!! block.createTextLine(line4, 1000);");
            block.createTextLine(line4, 1000);

            trace("!! block.releaseLines(line3, line5);");
            try {
                block.releaseLines(line3, line5);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("!! block.releaseLines(null, line5);");
            try {
                block.releaseLines(null, line5);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("!! block.releaseLines(line5, null);");
            try {
                block.releaseLines(line5, null);
            } catch(e:Error) {
                trace(e.getStackTrace());
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

            block.recreateTextLine(line5, line4, 1000);
            line5.y = 120;

            trace("!!!! reset all lines");

            trace("!! block.releaseLines(line5, line4);");
            try {
                block.releaseLines(line5, line4);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            dumpInfo(block, linesList);

            trace("!! block.releaseLines(line3, line1);");
            try {
                block.releaseLines(line3, line1);
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

            block.recreateTextLine(line5, line4, 1000);
            line5.y = 120;

            trace("!!!! reset all lines");

            dumpInfo(block, linesList);

            trace("!! block.releaseLines(line2, line5);");
            block.releaseLines(line2, line5);

            dumpInfo(block, linesList);

            trace("!! block.releaseLines(line0, line1);");
            block.releaseLines(line0, line1);

            dumpInfo(block, linesList);

            // At this point every single line should be invalid, let's see what
            // happens if we try to remove again

            trace("!! block.releaseLines(line1, line3);");
            try {
                block.releaseLines(line1, line3);
            } catch(e:Error) {
                trace(e.getStackTrace());
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

            block.recreateTextLine(line5, line4, 1000);
            line5.y = 120;

            trace("!!!! reset all lines");

            trace("!! block.releaseLines(line4, line4);");
            block.releaseLines(line4, line4);

            dumpInfo(block, linesList);

            trace("!! block.releaseLines(line4, line5);");
            try {
                block.releaseLines(line4, line5);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("!! block.releaseLines(line3, line4);");
            try {
                block.releaseLines(line3, line4);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            try {
            trace("!! block.releaseLines(line4, line3);");
                block.releaseLines(line4, line3);
            } catch(e:Error) {
                trace(e.getStackTrace());
            }

            trace("!! block.releaseLines(line3, line5);");
            block.releaseLines(line3, line5);

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

            trace("!!!! reset all lines");

            trace("!! line1.validity = \"static\";");
            line1.validity = "static";

            trace("!! line2.validity = \"static\";");
            line2.validity = "static";

            dumpInfo(block, linesList);

            trace("!! block.releaseLines(line1, line2);");
            block.releaseLines(line1, line2);

            dumpInfo(block, linesList);

            trace("!! block.releaseLines(line0, line4);");
            block.releaseLines(line0, line4);

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

            trace("!!!! reset all lines");

            var content2:ContentElement = new TextElement("Oremlay ipsumyay olorday\nitsay ametway, onsecteturcay adipiscingyay elitway, edsay");
            content2.elementFormat = new ElementFormat();
            var block2:TextBlock = new TextBlock(content2);

            trace("!!!! Created a completely different text block, block2");

            trace("!! block2.releaseLines(line1, line4);");
            try {
                block2.releaseLines(line1, line4);
            } catch(e:Error) {
                trace(e.getStackTrace());
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

            block.recreateTextLine(line5, line4, 1000);
            line5.y = 120;

            trace("!!!! reset all lines");

            trace("!! line4.validity = \"static\";");
            line4.validity = "static";

            trace("!! line5.validity = \"static\";");
            line5.validity = "static";

            dumpInfo(block, linesList);

            trace("!! block.releaseLines(line2, line4);");
            block.releaseLines(line2, line4);

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
