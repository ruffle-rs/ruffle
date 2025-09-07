// mxmlc -o test.swf -debug Test.as
package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

import flash.text.engine.SpaceJustifier;
var sj:SpaceJustifier = new SpaceJustifier();
sj.minimumSpacing = 1;
sj.maximumSpacing = 2;
sj.optimumSpacing = 1;

var sj2:SpaceJustifier = sj.clone() as SpaceJustifier;

trace("trace(sj.locale == sj2.locale);");
trace(sj.locale == sj2.locale);
trace("trace(sj.lineJustification == sj2.lineJustification);");
trace(sj.lineJustification == sj2.lineJustification);
trace("trace(sj.letterSpacing == sj2.letterSpacing);");
trace(sj.letterSpacing == sj2.letterSpacing);
trace("trace(sj.minimumSpacing == sj2.minimumSpacing);");
trace(sj.minimumSpacing == sj2.minimumSpacing);
trace("trace(sj.maximumSpacing == sj2.maximumSpacing);");
trace(sj.maximumSpacing == sj2.maximumSpacing);
trace("trace(sj.optimumSpacing == sj2.optimumSpacing);");
trace(sj.optimumSpacing == sj2.optimumSpacing);
