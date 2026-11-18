// FTE only uses DefineFont4 fonts, which mxmlc can't create from a TTF, so the
// font is first converted with the AIR SDK's fontswf:
//
//   fontswf -4 -a SpaceAB -u U+0020,U+0061-0062 -o spaceab.swf SpaceAB.ttf
//   mxmlc -static-link-runtime-shared-libraries -omit-trace-statements=false Test.as -o test.swf
package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.text.engine.ElementFormat;
    import flash.text.engine.FontDescription;
    import flash.text.engine.FontLookup;
    import flash.text.engine.TextBlock;
    import flash.text.engine.TextElement;
    import flash.text.engine.TextLine;

    public class Test extends Sprite {
        [Embed(source="spaceab.swf", fontName="SpaceAB")]
        private var SpaceAB:Class;

        private var recreated:TextLine;
        private var fresh:TextLine;

        public function Test() {
            var format:ElementFormat = new ElementFormat(
                new FontDescription("SpaceAB", "normal", "normal", FontLookup.EMBEDDED_CFF),
                20
            );
            var text:String = "abab abab abab abab abab abab";

            var block:TextBlock = new TextBlock(new TextElement("ab", format));
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
            trace("rawTextLength: " + recreated.rawTextLength + " " + fresh.rawTextLength);

            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function onEnterFrame(event:Event):void {
            removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            trace("width: " + recreated.width + " " + fresh.width);
            trace("height: " + recreated.height + " " + fresh.height);
        }
    }
}
