// mxmlc -o test.swf -debug Test.as
package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

import flash.text.engine.EastAsianJustifier;
var eaj:EastAsianJustifier = new EastAsianJustifier();

var eaj2:EastAsianJustifier = eaj.clone() as EastAsianJustifier;

trace("trace(eaj.locale == eaj2.locale);");
trace(eaj.locale == eaj2.locale);
trace("trace(eaj.lineJustification == eaj2.lineJustification);");
trace(eaj.lineJustification == eaj2.lineJustification);
trace("trace(eaj.justificationStyle == eaj2.justificationStyle);");
trace(eaj.justificationStyle == eaj2.justificationStyle);
trace("trace(eaj.composeTrailingIdeographicSpaces == eaj2.composeTrailingIdeographicSpaces);");
trace(eaj.composeTrailingIdeographicSpaces == eaj2.composeTrailingIdeographicSpaces);
