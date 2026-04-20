package {

import flash.display.Sprite;
import flash.utils.*;

public class Test extends Sprite {
    public function Test() {
        testNumberSerialization(0);
        testNumberSerialization(3);
    }

    private function testNumberSerialization(oe:int) {
        var ba = new ByteArray();
        ba.objectEncoding = oe;

        ba.writeObject(3);
        ba.writeObject(-5);
        ba.writeObject(0);
        ba.writeObject(1/Number.POSITIVE_INFINITY);
        ba.writeObject(1/Number.NEGATIVE_INFINITY);
        ba.writeObject(1073741824);
        ba.writeObject(-1073741824);
        ba.writeObject(0.5);
        ba.writeObject(getHalf() + getHalf());

        trace("AMF" + oe);
        ba.position = 0;
        while (ba.position < ba.length) {
            trace(ba.readByte());
        }
    }

    private function getHalf(): Number {
        return 0.5;
    }
}

}
