package {
    import flash.display.MovieClip;
    import flash.text.engine.*;

    public class Test extends MovieClip {
        public function Test() {
            var content:ContentElement = new TextElement("Some text here", new ElementFormat());

            // Just a regular block.
            var block:TextBlock = new TextBlock(content);

            // Another regular block.
            var block2:TextBlock = new TextBlock(content);

            // A block with `null` content.
            var blockEmpty:TextBlock = new TextBlock(null);

            // A block with `null` `content.elementFormat`.
            var blockFormatless:TextBlock = new TextBlock(new TextElement("Some text"));

            try {
                var result:TextLine = blockEmpty.createTextLine(null);
                trace("blockEmpty.createTextLine(null): " + result);
            } catch(e:Error) {
                trace("blockEmpty.createTextLine(null): error " + e.getStackTrace());
            }

            try {
                var result:TextLine = blockEmpty.createTextLine(null, -1);
                trace("blockEmpty.createTextLine(null, -1): " + result);
            } catch(e:Error) {
                trace("blockEmpty.createTextLine(null, -1): error " + e.getStackTrace());
            }

            try {
                var result:TextLine = block.createTextLine(null, -1);
                trace("block.createTextLine(null, -1): " + result);
            } catch(e:Error) {
                trace("block.createTextLine(null, -1): error " + e.getStackTrace());
            }

            try {
                var result:TextLine = block.createTextLine(null, 1000001);
                trace("block.createTextLine(null, 1000001): " + result);
            } catch(e:Error) {
                trace("block.createTextLine(null, 1000001): error " + e.getStackTrace());
            }

            var invalidLine:TextLine = block.createTextLine(null);
            invalidLine.validity = "invalid";

            try {
                var result:TextLine = block.createTextLine(invalidLine);
                trace("block.createTextLine(invalidLine): " + result);
            } catch(e:Error) {
                trace("block.createTextLine(invalidLine): error " + e.getStackTrace());
            }

            var line2:TextLine = block2.createTextLine(null);

            try {
                var result:TextLine = block.createTextLine(line2);
                trace("block.createTextLine(line2): " + result);
            } catch(e:Error) {
                trace("block.createTextLine(line2): error " + e.getStackTrace());
            }

            try {
                var result:TextLine = blockFormatless.createTextLine(null);
                trace("blockFormatless.createTextLine(null): " + result);
            } catch(e:Error) {
                trace("blockFormatless.createTextLine(null): error " + e.getStackTrace());
            }
        }
    }
}
