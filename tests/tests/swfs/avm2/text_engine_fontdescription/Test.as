package {
    import flash.display.Sprite;
    import flash.text.engine.*;

    public class Test extends Sprite {
        public function Test():void {
            var font = new FontDescription();

            var properties = ["cffHinting", "fontLookup", "fontName", "fontPosture", "fontWeight", "renderingMode"];
            for each (var prop in properties) {
                trace("// FontDescription::" + prop)

                try {
                    font[prop] = null;
                } catch (e) {
                    trace(e);
                }

                try {
                    font[prop] = "<invalid>";
                } catch (e) {
                    trace(e);
                }
            }

            font.cffHinting = CFFHinting.NONE;
            trace("// cffHinting: " + font.cffHinting);
            font.cffHinting = CFFHinting.HORIZONTAL_STEM;
            trace("// cffHinting: " + font.cffHinting);

            font.fontLookup = FontLookup.DEVICE;
            trace("// fontLookup: " + font.fontLookup);
            font.fontLookup = FontLookup.EMBEDDED_CFF;
            trace("// fontLookup: " + font.fontLookup);

            font.fontPosture = FontPosture.NORMAL;
            trace("// fontPosture: " + font.fontPosture);
            font.fontPosture = FontPosture.ITALIC;
            trace("// fontPosture: " + font.fontPosture);

            font.fontWeight = FontWeight.NORMAL;
            trace("// fontWeight: " + font.fontWeight);
            font.fontWeight = FontWeight.BOLD;
            trace("// fontWeight: " + font.fontWeight);

            font.renderingMode = RenderingMode.NORMAL;
            trace("// renderingMode: " + font.renderingMode);
            font.renderingMode = RenderingMode.CFF;
            trace("// renderingMode: " + font.renderingMode);
        }
    }
}