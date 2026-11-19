package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.text.engine.ElementFormat;
    import flash.text.engine.FontDescription;
    import flash.text.engine.TextBlock;
    import flash.text.engine.TextElement;
    import flash.text.engine.TextLine;

    public class Test extends Sprite {
        private var recreated:TextLine;
        private var fresh:TextLine;

        public function Test() {
            var format:ElementFormat = new ElementFormat(new FontDescription("_sans"), 20);
            var text:String = "BRING ALL PAINT-GUYS TOGETHER";

            var block:TextBlock = new TextBlock(new TextElement("GO", format));
            var line:TextLine = block.createTextLine(null, 40);

            block.content = new TextElement(text, format);
            recreated = block.recreateTextLine(line, null, 1000);

            var freshBlock:TextBlock = new TextBlock(new TextElement(text, format));
            fresh = freshBlock.createTextLine(null, 1000);

            recreated.y = 50;
            fresh.y = 150;
            addChild(recreated);
            addChild(fresh);

            trace("recreated is the same object: " + (recreated === line));
            trace("specifiedWidth: " + recreated.specifiedWidth);
            trace("same rawTextLength: " + (recreated.rawTextLength == fresh.rawTextLength));

            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function onEnterFrame(event:Event):void {
            removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            trace("same width: " + (recreated.width == fresh.width));
            trace("same height: " + (recreated.height == fresh.height));
        }
    }
}
