package {
    [Ruffle(CustomConstructor)]
    [Ruffle(CallHandler)]
    public final dynamic class Function {
        private static native function _initFunctionClass():void;

        {
            _initFunctionClass();

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
