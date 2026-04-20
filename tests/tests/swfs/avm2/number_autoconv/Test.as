package {

import flash.display.Sprite;
import avmplus.getQualifiedClassName;
import flash.utils.describeType;

public class Test extends Sprite {
    public function Test() {
        trace(getQualifiedClassName(getV()));
        trace(getQualifiedClassName(getV() + getV()));
        trace(getQualifiedClassName(getV() * 2));
        trace(getQualifiedClassName(getV() * getV()));
        trace(getQualifiedClassName(getU()));

        trace("// int overflow")
        trace(getQualifiedClassName(getW()));
        trace(getQualifiedClassName(getW() + 1.0));
        trace(getQualifiedClassName(getW() + 2.0));
        trace("// int underflow")
        trace(getQualifiedClassName(-getW()));
        trace(getQualifiedClassName(-getW() - 1.0));
        trace(getQualifiedClassName(-getW() - 2.0));
        trace(getQualifiedClassName(-getW() - 3.0));

        trace("// should work the same for describeType")
        trace(describeType(getV() + getV()));
    }

    private function getU(): Number {
        return 1.0;
    }

    private function getV(): Number {
        return 1.5;
    }

    private function getW(): Number {
        return 268435454.0;
    }
}

}
