// Compile with:
//   mxmlc -static-link-runtime-shared-libraries -output test.swf Test.as
package {
    import flash.display.BlendMode;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.filters.GlowFilter;
    import flash.text.TextField;
    import flash.text.TextFormat;

    [SWF(width="400", height="380", backgroundColor="#FFFFFF", frameRate="30")]
    public class Test extends Sprite {
        [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0020,U+0061-U+0064")]
        private var testFont:Class;

        private var fields:Array = [];

        public function Test() {
            stage.scaleMode = "noScale";
            stage.align = "TL";

            graphics.beginFill(0xFFCC00);
            graphics.drawRect(0, 300, 400, 50);
            graphics.endFill();

            add(10, 0x000000, false, 0);
            add(70, 0x000000, false, 0);
            var focused:TextField = add(130, 0xFFFFFF, true, 0x202020);
            add(190, 0xFF0000, false, 0);
            add(250, 0x000000, false, 0).filters = [new GlowFilter(0x00FF00, 1, 8, 8, 4)];
            add(310, 0x000000, false, 0).blendMode = BlendMode.MULTIPLY;

            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                stage.focus = focused;
                for each (var field:TextField in fields) {
                    field.setSelection(2, 9);
                }
            });
        }

        private function add(y:Number, color:uint, background:Boolean, backgroundColor:uint):TextField {
            var field:TextField = new TextField();
            field.x = 10;
            field.y = y;
            field.width = 380;
            field.height = 44;
            field.embedFonts = true;
            field.defaultTextFormat = new TextFormat("TestFont", 30, color);
            field.text = "abcd abcd abcd";
            field.background = background;
            field.backgroundColor = backgroundColor;
            field.alwaysShowSelection = true;
            field.type = "input";
            addChild(field);
            fields.push(field);
            return field;
        }
    }
}
