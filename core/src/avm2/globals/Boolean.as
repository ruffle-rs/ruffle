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
                Error.throwError(TypeError, 1004, "Boolean.prototype.toString");
            }

            return this.AS3::toString();
        };

        prototype.valueOf = function():* {
            if (this === Boolean.prototype) {
                return false;
            }

            if (!(this is Boolean)) {
                Error.throwError(TypeError, 1004, "Boolean.prototype.valueOf");
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
