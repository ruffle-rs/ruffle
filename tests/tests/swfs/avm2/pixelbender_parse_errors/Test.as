package {
    import flash.display.ShaderData;
    import flash.display.Sprite;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        public function Test() {
            super();
            var theArray:ByteArray = new ByteArray();
            try {
                new ShaderData(theArray);
                trace();
            } catch(e:Error) {
                trace(Object.prototype.toString.call(e));
                trace(e.errorID);
            }
            theArray.writeByte(0);
            try {
                new ShaderData(theArray);
            } catch(e:Error) {
                trace(Object.prototype.toString.call(e));
                trace(e.errorID);
            }
            theArray[0] = 0xA4;
            try {
                new ShaderData(theArray);
            } catch(e:Error) {
                trace(Object.prototype.toString.call(e));
                trace(e.errorID);
            }
        }
    }
}
