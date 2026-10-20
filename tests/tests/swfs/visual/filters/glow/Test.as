package {
    import flash.display.DisplayObject;
    import flash.display.Sprite;
    import flash.display.StageAlign;
    import flash.display.StageQuality;
    import flash.display.StageScaleMode;
    import flash.filters.BitmapFilterQuality;
    import flash.filters.GlowFilter;
    import flash.text.AntiAliasType;
    import flash.text.Font;
    import flash.text.TextField;
    import flash.text.TextFieldAutoSize;
    import flash.text.TextFormat;

    [SWF(width="550", height="400", frameRate="24", backgroundColor="#FFFFFF")]
    public class Test extends Sprite {
        [Embed(source="shape.swf", symbol="GlowShape")]
        private static const OriginalShape:Class;

        [Embed(source="NotoSans.ttf", fontName="GlowLabels",
               embedAsCFF="false", unicodeRange="U+002C,U+0049,U+004B")]
        private static const LabelFont:Class;

        public function Test() {
            stage.scaleMode = StageScaleMode.NO_SCALE;
            stage.align = StageAlign.TOP_LEFT;
            stage.quality = StageQuality.LOW;
            Font.registerFont(LabelFont);

            // Original placements: outer, outer knockout, inner, inner knockout.
            addCase(37.05, 8.95, false, false);
            addCase(298, 13.95, false, true);
            addCase(32, 204.5, true, false);
            addCase(285, 209.5, true, true);

            addLabel("K", 495.9, 19.75);
            addLabel("I", 209.05, 206.5);
            addLabel("K,I", 493, 214.75);
        }

        private function addCase(x:Number, y:Number, inner:Boolean, knockout:Boolean):void {
            var shape:DisplayObject = new OriginalShape();
            shape.x = x;
            shape.y = y;
            shape.filters = [new GlowFilter(
                0xFF0000, 1, 5, 5, 1, BitmapFilterQuality.LOW, inner, knockout
            )];
            addChild(shape);
        }

        private function addLabel(value:String, x:Number, y:Number):void {
            var label:TextField = new TextField();
            label.embedFonts = true;
            label.selectable = false;
            label.antiAliasType = AntiAliasType.NORMAL;
            label.autoSize = TextFieldAutoSize.LEFT;
            label.defaultTextFormat = new TextFormat("GlowLabels", 29, 0x000000);
            label.text = value;
            label.x = x;
            label.y = y;
            addChild(label);
        }
    }
}
