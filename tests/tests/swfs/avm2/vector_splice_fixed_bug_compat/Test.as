// compiled with mxmlc

import flash.utils.getQualifiedClassName;
import flash.utils.getTimer;
import flash.utils.ByteArray;

var v = new Vector.<int>(5, true);
trace(v.length);
try {
    v.push(123);
} catch(e) {
    trace("push exception caught");
}
v.splice(0, 3);
trace("NO exception on splice")
trace(v.length);

package {
    import flash.display.MovieClip;
    import flash.text.TextField;

    public class Test extends MovieClip {
        public function Test(){

        }
    }
}

