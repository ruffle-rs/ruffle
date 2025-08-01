package {
    import flash.display.Sprite;

    public class SupercallTest extends Sprite {

        public var varSlot1:int = 3;

        public var varSlot2:int = 4;

        public const constSlot:int = 5;

        public function SupercallTest() {
            super();
            trace("SupercallTest() called");
        }

        public function get getter() : Function {
            trace("SuperCallTest.getter called");
            return function():* {
                trace("function returned by getter called");
            };
        }

        public function set setter(value:int) : void {
            trace("SuperCallTest.setter called");
        }

        public function method() : void {
            trace("SuperCallTest.method called");
        }
    }
}
