package {
    import flash.display.Sprite;
    import flash.text.engine.ElementFormat;

    public class Test extends Sprite {
        public function Test() {
            var format:ElementFormat = new ElementFormat();

            trace("fontDescription.fontName: " + format.fontDescription.fontName);
            trace("fontDescription.fontWeight: " + format.fontDescription.fontWeight);
            trace("fontDescription.fontPosture: " + format.fontDescription.fontPosture);
            trace("fontDescription.fontLookup: " + format.fontDescription.fontLookup);
            trace("fontDescription.renderingMode: " + format.fontDescription.renderingMode);
            trace("fontDescription.cffHinting: " + format.fontDescription.cffHinting);
            trace("fontDescription.locked: " + format.fontDescription.locked);
            trace("fontSize: " + format.fontSize);
            trace("color: " + format.color);
            trace("alpha: " + format.alpha);
            trace("textRotation: " + format.textRotation);
            trace("dominantBaseline: " + format.dominantBaseline);
            trace("alignmentBaseline: " + format.alignmentBaseline);
            trace("baselineShift: " + format.baselineShift);
            trace("kerning: " + format.kerning);
            trace("trackingRight: " + format.trackingRight);
            trace("trackingLeft: " + format.trackingLeft);
            trace("locale: " + format.locale);
            trace("breakOpportunity: " + format.breakOpportunity);
            trace("digitCase: " + format.digitCase);
            trace("digitWidth: " + format.digitWidth);
            trace("ligatureLevel: " + format.ligatureLevel);
            trace("typographicCase: " + format.typographicCase);
            trace("locked: " + format.locked);
        }
    }
}
