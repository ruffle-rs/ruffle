package {
import flash.display.*;

public class Test extends MovieClip {
    public function Test() {
        testString(String.fromCharCode(0xDC00));
        testString(String.fromCharCode(0xDC00, 0xDFFF));
        testString(String.fromCharCode(0xDC00, 0xABAB));
        testString(String.fromCharCode(0xDE00, 0xABAB));
    }

    private function testString(s) {
        try {
            trace(encodeURI(s));
        } catch (e) {
            trace(e);
        }
        try {
            trace(encodeURIComponent(s));
        } catch (e) {
            trace(e);
        }
    }
}
}
