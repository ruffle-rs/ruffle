// mxmlc -o test.swf -debug Test.as
package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

import flash.text.engine.FontDescription;
var fd:FontDescription = new FontDescription();
fd.locked = true;

var fd2:FontDescription = fd.clone();

trace("trace(fd.fontName == fd2.fontName);");
trace(fd.fontName == fd2.fontName);
trace("trace(fd.fontWeight == fd2.fontWeight);");
trace(fd.fontWeight == fd2.fontWeight);
trace("trace(fd.fontPosture == fd2.fontPosture);");
trace(fd.fontPosture == fd2.fontPosture);
trace("trace(fd.fontLookup == fd2.fontLookup);");
trace(fd.fontLookup == fd2.fontLookup);
trace("trace(fd.renderingMode == fd2.renderingMode);");
trace(fd.renderingMode == fd2.renderingMode);
trace("trace(fd.cffHinting == fd2.cffHinting);");
trace(fd.cffHinting == fd2.cffHinting);
trace("trace(fd.locked == fd2.locked);");
trace(fd.locked == fd2.locked);
