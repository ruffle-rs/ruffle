package {
    [Ruffle(CustomConstructor)]
    [Ruffle(CallHandler)]
    public final dynamic class Function {
        private static native function _initFunctionClass():void;
        private static native function _initFunctionProto():void;

        [Ruffle(NativeAccessible)]
        private static var dummyFunction:Function = null;

        {
            // These steps need to be done in order. We absolutely need the
            // Function class to be in SystemClasses for the `newfunction` op
            // to work; once we can create a function using `newfunction`, we
            // can store the dummy function. Function's prototype is a dummy
            // function, so we can only initialize Function's prototype once
            // we have a dummy function on hand.
            _initFunctionClass();
            dummyFunction = function() {};
            _initFunctionProto();

            // Initialize prototype functions on Object
            Object.init();

            // Initialize our prototype functions
            prototype.apply = function(receiver:* = void 0, args:* = void 0):* {
                var f:Function = this;
                return f.AS3::apply(receiver, args);
            };

            prototype.call = function(receiver:* = void 0, ...rest):* {
                // `rest` is passed as an Array, so we can use `Function.apply`
                var f:Function = this;
                return f.AS3::apply(receiver, rest);
            };

            prototype.toString = function():String {
                return "function Function() {}";
            };

            prototype.toLocaleString = prototype.toString;

            prototype.setPropertyIsEnumerable("apply", false);
            prototype.setPropertyIsEnumerable("call", false);
            prototype.setPropertyIsEnumerable("toString", false);
            prototype.setPropertyIsEnumerable("toLocaleString", false);
        }

        public function Function() {
            // The Function constructor is implemented natively:
            // this AS-defined method does nothing
        }

        public native function get length() : int;

        public native function get prototype():*;
        public native function set prototype(proto:*):*;

        AS3 native function call(receiver:* = void 0, ...rest):*;

        AS3 native function apply(receiver:* = void 0, args:* = void 0):*;

        public static const length:int = 1;
    }
}
