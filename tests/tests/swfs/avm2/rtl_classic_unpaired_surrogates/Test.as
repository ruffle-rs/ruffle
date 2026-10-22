package {
    import flash.display.Sprite;
    import flash.geom.Rectangle;
    import flash.text.TextField;
    import flash.text.TextFormat;

    [SWF(width="620", height="220", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            var samples:Array = [
                String["fromCharCode"](0x05D0, 0x0041),
                String["fromCharCode"](0x05D0, 0xD800, 0x0041),
                String["fromCharCode"](0x05D0, 0xDC00, 0x0041),
                String["fromCharCode"](0x05D0, 0xD83D, 0xDE00, 0x0041)
            ];

            for (var i:int = 0; i < samples.length; i++) {
                var field:TextField = new TextField();
                field.x = 20;
                field.y = 20 + i * 45;
                field.width = 550;
                field.height = 40;
                field.border = true;
                field.selectable = false;
                field.defaultTextFormat =
                    new TextFormat("DejaVu Sans", 22, 0x000000);
                trace("source=" + i + " length=" + samples[i].length);
                for (var k:int = 0; k < samples[i].length; k++) {
                    trace("  sourceIndex=" + k +
                        " code=" + samples[i].charCodeAt(k));
                }
                field.text = samples[i];
                addChild(field);

                trace("case=" + i + " length=" + field.length);
                for (var j:int = 0; j < field.length; j++) {
                    var bounds:Rectangle = field.getCharBoundaries(j);
                    trace("  index=" + j +
                        " code=" + field.text.charCodeAt(j) +
                        " hasBounds=" + (bounds != null));
                    if (bounds != null &&
                        bounds.width > 0 && bounds.height > 0) {
                        trace("    hit=" + field.getCharIndexAtPoint(
                            bounds.x + bounds.width / 2,
                            bounds.y + bounds.height / 2));
                    }
                }
            }
            trace("done");
        }
    }
}
