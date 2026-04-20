package {
    [Ruffle(CustomConstructor)]
    [Ruffle(CallHandler)]
    public final class Boolean {
        public function Boolean(value:* = void 0) {
            // The Boolean constructor is implemented natively:
            // this AS-defined method does nothing
        }

        prototype.toString = function():String {
            if (this === Boolean.prototype) {
                return "false";
            }

            if (!(this is Boolean)) {
                throw new TypeError("Error #1004: Method Boolean.prototype.toString was invoked on an incompatible object.", 1004);
            }

            return this.AS3::toString();
        };

        prototype.valueOf = function():* {
            if (this === Boolean.prototype) {
                return false;
            }

            if (!(this is Boolean)) {
                throw new TypeError("Error #1004: Method Boolean.prototype.valueOf was invoked on an incompatible object.", 1004);
            }

            return this;
        };

        prototype.setPropertyIsEnumerable("toString", false);
        prototype.setPropertyIsEnumerable("valueOf", false);

        AS3 function toString():String {
            if (this) {
                return "true";
            } else {
                return "false";
            }
        }

        AS3 function valueOf():Boolean {
            return this;
        }

        public static const length:int = 1;
    }
}
