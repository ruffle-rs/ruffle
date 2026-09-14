package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {

        public var field:int;

        public function Test() {
            for(var i = 1; i <= 8; i ++) {
                trace("Case #" + i);
                try {
                    this["case" + 1]();
                } catch(e:Error) {
                    trace(e);
                }
                trace("");
            }
        }

        public function case1():void {
            // p-code test: setlocal0 using `inclocal`, then use it for scope,
            // property lookup, etc
        }

        public function case2():void {
            // p-code test: setlocal0 using `inclocali`, then use it for scope,
            // property lookup, etc
        }

        public function case3():void {
            // p-code test: setlocal0 using `declocal`, then use it for scope,
            // property lookup, etc
        }

        public function case4():void {
            // p-code test: setlocal0 using `declocali`, then use it for scope,
            // property lookup, etc
        }

        public function case5():void {
            // p-code test: setlocal0 using `kill`, then use it for scope,
            // property lookup, etc
        }

        public function case6():void {
            // p-code test: setlocal0 using `hasnext2` (object register), then
            // use it for scope, property lookup, etc
        }

        public function case7():void {
            // p-code test: setlocal0 using `hasnext2` (index register), then
            // use it for scope, property lookup, etc
        }

        public function case8():void {
            // p-code test: setlocal0 after initial getlocal0+pushscope, then
            // use it for scope, property lookup, etc
        }
    }
}
