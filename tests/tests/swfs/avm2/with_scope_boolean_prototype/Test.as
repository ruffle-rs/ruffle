package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            Boolean.prototype.prop = "value";
            try {
                with (true) {
                    trace(prop);
                }
            } catch (e:ReferenceError) {
                trace("ReferenceError #" + e.errorID);
            }
        }
    }
}
