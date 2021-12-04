// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }

    }

}

var f = new Function();
trace(f)
trace(f())
var p = Function.prototype;
trace(p());
trace(Function.prototype.call.call());
