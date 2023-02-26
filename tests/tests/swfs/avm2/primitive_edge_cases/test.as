// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

var vec = Vector.<Object>([true, false]);
vec = Vector.<Boolean>(vec);
trace(vec);
