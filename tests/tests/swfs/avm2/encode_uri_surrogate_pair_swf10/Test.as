package {
import flash.display.*;

public class Test extends MovieClip {
    public function Test() {
        trace(encodeURI(String.fromCharCode(0xD800, 0xDC00)));
        trace(encodeURI(String.fromCharCode(0xD800, 0xDC01)));
        trace(encodeURI(String.fromCharCode(0xD801, 0xDC01)));
        trace(encodeURI(String.fromCharCode(0xD801, 0xDC02)));
        trace(encodeURI(String.fromCharCode(0xD842, 0xDF9F)));
        trace(encodeURI(String.fromCharCode(0xD842, 0xDFA0)));
        trace(encodeURI(String.fromCharCode(0xD8FA, 0xDD12)));
        trace(encodeURI(String.fromCharCode(0xD8FA, 0xDD13)));
        trace(encodeURI(String.fromCharCode(0xD8FA, 0xDF22)));
        trace(encodeURI(String.fromCharCode(0xD8FA, 0xDF23)));
        trace(encodeURI(String.fromCharCode(0xDA04, 0xDD21)));
        trace(encodeURI(String.fromCharCode(0xDA04, 0xDD22)));
        trace(encodeURI(String.fromCharCode(0xDBFE, 0xDFFE)));
        trace(encodeURI(String.fromCharCode(0xDBFF, 0xDFFE)));
        trace(encodeURI(String.fromCharCode(0xDBFF, 0xDFFF)));
    }
}
}
