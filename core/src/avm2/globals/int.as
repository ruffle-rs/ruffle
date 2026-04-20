package {
    [Ruffle(CustomConstructor)]
    [Ruffle(CallHandler)]
    public final class int {
        public static const MIN_VALUE:int = -2147483648;

        public static const MAX_VALUE:int = 2147483647;

        {
            prototype.toExponential = function(digits:* = 0):String {
                var self:Number = this;
                return self.AS3::toExponential(digits);
            };

            prototype.toFixed = function(digits:* = 0):String {
                var self:Number = this;
                return self.AS3::toFixed(digits);
            };

            prototype.toPrecision = function(digits:* = 0):String {
                var self:Number = this;

                if (digits == undefined) {
                    return self.AS3::toString();
                }

                return self.AS3::toPrecision(digits);
            };

            prototype.toString = function(radix:* = 10):String {
                if (this === int.prototype) {
                    return "0";
                }

                if (!(this is int)) {
                    throw new TypeError("Error #1004: Method int.prototype.toString was invoked on an incompatible object.", 1004);
                }

                var self:Number = this;
                return self.AS3::toString(radix);
            };

            prototype.toLocaleString = prototype.toString;

            prototype.valueOf = function():* {
                if (this === int.prototype) {
                    return 0;
                }

                if (!(this is int)) {
                    throw new TypeError("Error #1004: Method int.prototype.valueOf was invoked on an incompatible object.", 1004);
                }

                return this;
            };

            prototype.setPropertyIsEnumerable("toExponential", false);
            prototype.setPropertyIsEnumerable("toFixed", false);
            prototype.setPropertyIsEnumerable("toPrecision", false);
            prototype.setPropertyIsEnumerable("toString", false);
            prototype.setPropertyIsEnumerable("toLocaleString", false);
            prototype.setPropertyIsEnumerable("valueOf", false);
        }

        public function int(value:* = 0) {
            // The int constructor is implemented natively:
            // this AS-defined method does nothing
        }

        // These methods are unreachable. Calling a method on an Integer will
        // lookup and call the method using `Number`'s vtable, not `int`'s.
        // IMPORTANT: these methods must be kept in the same order as they are
        // declared in Number.as. Otherwise the bytecode optimizer may emit a
        // call to the wrong method when calling on an `int` or `uint`.
        AS3 native function toExponential(digits:* = 0):String;

        AS3 native function toFixed(digits:* = 0):String;

        AS3 native function toPrecision(digits:* = 0):String;

        AS3 native function toString(radix:* = 10):String;

        AS3 native function valueOf():int;

        public static const length:int = 1;
    }
}

