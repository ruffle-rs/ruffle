// compiled with mxmlc

import flash.display.*;
import flash.text.engine.TextLine;
import flash.system.Worker;

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
            var classes = [
                Stage,
                DisplayObject,
                DisplayObjectContainer,
                InteractiveObject,
                LoaderInfo,
                TextLine,
                Class,
                Math,
                MorphShape,
                Worker,
            ]
            for each(var t in classes) {
                trace(t);
                try {
                    new t();
                } catch(e) {
                    trace(e.getStackTrace());
                }
            }
        }
    }
}