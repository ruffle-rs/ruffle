package {
    import flash.display.Sprite;
    import flash.text.Font;

    public class Test extends Sprite {
        [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0061")]
        private var TestFont:Class;

        public function Test() {
            var a:Array = Font.enumerateFonts(false);
            var b:Array = Font.enumerateFonts(false);

            trace("// a.length");
            trace(a.length);
            trace("");

            trace("// b.length");
            trace(b.length);
            trace("");

            trace("// a[0] === b[0]");
            trace(a[0] === b[0]);

            trace("// a[0].fontName === b[0].fontName");
            trace(a[0].fontName === b[0].fontName);
            trace("");

            trace("// a[0] is TestFont");
            trace(a[0] is TestFont);
            trace("");

            Font.registerFont(TestFont);

            var c:Array = Font.enumerateFonts(false);

            trace("// c.length");
            trace(c.length);
            trace("");

            for (var i:int = 0; i < c.length; i++) {
                trace("// c[" + i + "].fontName");
                trace(c[i].fontName);
                trace("// c[" + i + "] is TestFont");
                trace(c[i] is TestFont);
                trace("");
            }
        }
    }
}
