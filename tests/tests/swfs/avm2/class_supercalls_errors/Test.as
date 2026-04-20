package {
    public class Test extends SupercallTest {

        public var varSlot2:int = 99;

        public function Test() {
            super();
            this.testsWithJit();
        }

        public function testsWithJit() : * {
            trace(this.getter());
            this.setter = 4;
            trace(this.method());
            trace(super.getter());
            super.setter = 4;
            trace(super.method());
            trace(this.varSlot1);
            trace(super.varSlot1);
            super.varSlot1 = 6;
            trace(this.varSlot1);
            trace(super.varSlot1);
            this.varSlot1 = 7;
            trace(this.varSlot1);
            trace(super.varSlot1);
            trace(this.varSlot2);
            trace(super.varSlot2);
            super.varSlot2 = 8;
            trace(this.varSlot2);
            trace(super.varSlot2);
            this.varSlot2 = 9;
            trace(this.varSlot2);
            trace(super.varSlot2);
            trace(super.method);
            try {
                this.constSlot = 55;
                trace("e1: no error");
            }
            catch(e:Error) {
                trace("e1: " + e.errorID);
            }
            try {
                super.constSlot = 55;
                trace("e2: no error");
            }
            catch(e:Error) {
                trace("e2: " + e.errorID);
            }
            try {
                super.nonexistent();
                trace("e3: no error");
            }
            catch(e:Error) {
                trace("e3: " + e.errorID);
            }
            try {
                trace(super.setter);
                trace("e4: no error");
            }
            catch(e:Error) {
                trace("e4: " + e.errorID);
            }
            try {
                super.getter = 4;
                trace("e5: no error");
            }
            catch(e:Error) {
                trace("e5: " + e.errorID);
            }
            try {
                trace(super.nonexistent);
                trace("e6: no error");
            }
            catch(e:Error) {
                trace("e6: " + e.errorID);
            }
        }

        override public function get getter() : Function {
            trace("Test.getter called");
            return super.getter;
        }

        override public function set setter(value:int) : void {
            trace("Test.setter called");
            super.setter = 3;
        }

        override public function method() : void {
            trace("Test.method called");
            super.method();
        }
    }
}
