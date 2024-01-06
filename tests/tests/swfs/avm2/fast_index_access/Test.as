package {
    import flash.display.Sprite;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        public function Test() {
            Object.prototype[1] = "at 1";
            Object.prototype[10] = "at 10";

            var array = new Array(2);
            array[0] = 1;
            trace("array[0]: " + array[0]);
            trace("array[1]: " + array[1]);
            trace("array[2]: " + array[2]);
            trace("array[10]: " + array[10]);

            var vector: Vector.<int> = new <int>[1, 2];
            trace("vector[0]: " + vector[0]);
            trace("vector[1]: " + vector[1]);
            try {
                trace("vector[2]: " + vector[2]);
            } catch (e) {
                trace("vector[2]: "+ e);
            }
            try {
                trace("vector[10]: " + vector[10]);
            } catch (e) {
                trace("vector[10]: " + e);
            }

            var byteArray = new ByteArray();
            byteArray.writeByte(1);
            byteArray.writeByte(2);
            trace("byteArray[0]: " + byteArray[0]);
            trace("byteArray[1]: " + byteArray[1]);
            try {
                trace("byteArray[2]: " + byteArray[2]);
            } catch (e) {
                trace("byteArray[2]: " + e);
            }
            try {
                trace("byteArray[10]: " + byteArray[10]);
            } catch (e) {
                trace("byteArray[10]: " + e);
            }
        }
    }
}
