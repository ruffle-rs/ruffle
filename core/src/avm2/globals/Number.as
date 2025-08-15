package {
    [Ruffle(CustomConstructor)]
    [Ruffle(CallHandler)]
    public final class Number {
        public static const MAX_VALUE:Number = 1.7976931348623157e+308;

        public static const MIN_VALUE:Number = 2.2250738585072014e-308;

        public static const NaN:Number = 0 / 0;

        public static const NEGATIVE_INFINITY:Number = -1 / 0;

        public static const POSITIVE_INFINITY:Number = 1 / 0;

        [API("680")]
        public static const E:Number = 2.718281828459045;

        [API("680")]
        public static const PI:Number = 3.141592653589793;

        [API("680")]
        public static const SQRT2:Number = 1.4142135623730951;

        [API("680")]
        public static const SQRT1_2:Number = 0.7071067811865476;

        [API("680")]
        public static const LN2:Number = 0.6931471805599453;

        [API("680")]
        public static const LN10:Number = 2.302585092994046;

        [API("680")]
        public static const LOG2E:Number = 1.4426950408889634;

        [API("680")]
        public static const LOG10E:Number = 0.4342944819032518;

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
                if (this === Number.prototype) {
                    return "0";
                }

                if (!(this is Number)) {
                    throw new TypeError("Error #1004: Method Number.prototype.toString was invoked on an incompatible object.", 1004);
                }

                var self:Number = this;
                return self.AS3::toString(radix);
            };

            prototype.toLocaleString = prototype.toString;

            prototype.valueOf = function():* {
                if (this === Number.prototype) {
                    return 0;
                }

                if (!(this is Number)) {
                    throw new TypeError("Error #1004: Method Number.prototype.valueOf was invoked on an incompatible object.", 1004);
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

        public function Number(value:* = 0) {
            // The Number constructor is implemented natively:
            // this AS-defined method does nothing
        }

        AS3 native function toExponential(digits:* = 0):String;

        AS3 native function toFixed(digits:* = 0):String;

        AS3 native function toPrecision(digits:* = 0):String;

        AS3 native function toString(radix:* = 10):String;

        AS3 native function valueOf():Number;

        public static const length:int = 1;
    }
}

