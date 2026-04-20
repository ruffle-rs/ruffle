package {
    [Ruffle(ConstructOnCall)]
    [Ruffle(CustomConstructor)]
    public final dynamic class Function {
        private static native function _initFunctionClass():void;

        {
            _initFunctionClass();

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

        public native function get length():int;

        [Ruffle(FastCall)]
        public native function get prototype():*;
        [Ruffle(FastCall)]
        public native function set prototype(proto:*):*;

        AS3 native function apply(receiver:* = void 0, args:* = void 0):*;
        AS3 native function call(receiver:* = void 0, ...rest):*;

        [Ruffle(NativeCallable)]
        private static function createDummyFunction():Function {
            return function() {};
        }

        public static const length:int = 1;
    }
}
