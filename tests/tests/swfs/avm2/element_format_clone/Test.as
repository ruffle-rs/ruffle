// mxmlc -o test.swf -debug Test.as
package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

import flash.text.engine.FontDescription;
import flash.text.engine.ElementFormat;
var ef:ElementFormat = new ElementFormat(new FontDescription());

var ef2:ElementFormat = ef.clone();

trace("trace(ef.fontDescription.fontName == ef2.fontDescription.fontName);");
trace(ef.fontDescription.fontName == ef2.fontDescription.fontName);
trace("trace(ef.fontDescription.fontWeight == ef2.fontDescription.fontWeight);");
trace(ef.fontDescription.fontWeight == ef2.fontDescription.fontWeight);
trace("trace(ef.fontDescription.fontPosture == ef2.fontDescription.fontPosture);");
trace(ef.fontDescription.fontPosture == ef2.fontDescription.fontPosture);
trace("trace(ef.fontDescription.fontLookup == ef2.fontDescription.fontLookup);");
trace(ef.fontDescription.fontLookup == ef2.fontDescription.fontLookup);
trace("trace(ef.fontDescription.renderingMode == ef2.fontDescription.renderingMode);");
trace(ef.fontDescription.renderingMode == ef2.fontDescription.renderingMode);
trace("trace(ef.fontDescription.cffHinting == ef2.fontDescription.cffHinting);");
trace(ef.fontDescription.cffHinting == ef2.fontDescription.cffHinting);

trace("trace(ef.fontSize == ef2.fontSize);");
trace(ef.fontSize == ef2.fontSize);
trace("trace(ef.color == ef2.color);");
trace(ef.color == ef2.color);
trace("trace(ef.alpha == ef2.alpha);");
trace(ef.alpha == ef2.alpha);
trace("trace(ef.textRotation == ef2.textRotation);");
trace(ef.textRotation == ef2.textRotation);
trace("trace(ef.dominantBaseline == ef2.dominantBaseline);");
trace(ef.dominantBaseline == ef2.dominantBaseline);
trace("trace(ef.alignmentBaseline == ef2.alignmentBaseline);");
trace(ef.alignmentBaseline == ef2.alignmentBaseline);
trace("trace(ef.baselineShift == ef2.baselineShift);");
trace(ef.baselineShift == ef2.baselineShift);
trace("trace(ef.kerning == ef2.kerning);");
trace(ef.kerning == ef2.kerning);
trace("trace(ef.trackingRight == ef2.trackingRight);");
trace(ef.trackingRight == ef2.trackingRight);
trace("trace(ef.trackingLeft == ef2.trackingLeft);");
trace(ef.trackingLeft == ef2.trackingLeft);
trace("trace(ef.locale == ef2.locale);");
trace(ef.locale == ef2.locale);
trace("trace(ef.breakOpportunity == ef2.breakOpportunity);");
trace(ef.breakOpportunity == ef2.breakOpportunity);
trace("trace(ef.digitCase == ef2.digitCase);");
trace(ef.digitCase == ef2.digitCase);
trace("trace(ef.digitWidth == ef2.digitWidth);");
trace(ef.digitWidth == ef2.digitWidth);
trace("trace(ef.ligatureLevel == ef2.ligatureLevel);");
trace(ef.ligatureLevel == ef2.ligatureLevel);
trace("trace(ef.typographicCase == ef2.typographicCase);");
trace(ef.typographicCase == ef2.typographicCase);
