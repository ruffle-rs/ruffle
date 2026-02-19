package {

import flash.display.Sprite;
import avmplus.*;

public class Test extends Sprite {
    public function Test() {
        trace(getQualifiedClassName(getV()));
        trace(getQualifiedClassName(getV() + getV()));
        trace(getQualifiedClassName(getV() * 2));
        trace(getQualifiedClassName(getV() * getV()));
        trace(getQualifiedClassName(getU()));
    }

    private function getU(): Number {
        return 1.0;
    }

    private function getV(): Number {
        return 1.5;
    }
}

}
