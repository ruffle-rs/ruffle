// compiled with mxmlc (and modified to SWF v9)

import flash.utils.ByteArray;
import flash.geom.Point;

var p = new Point(4.5, 5.5);
var b = new ByteArray();
b.writeObject(p);
trace(b.length);

package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            return;
        }
    }
}
