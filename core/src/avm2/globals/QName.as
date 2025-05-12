package {
    [Ruffle(CustomConstructor)]
    [Ruffle(CallHandler)]
    public final class QName {
        public static const length:* = 2;

        public function QName(uri:* = void 0, localName:* = void 0) {
            // The QName constructor is implemented natively:
            // this AS-defined method does nothing
        }

        public native function get localName():String;
        public native function get uri():*;

        AS3 native function toString():String;
        AS3 function valueOf():QName {
            return this;
        }

        prototype.toString = function():String {
            var self:QName = this;
            return self.AS3::toString();
        }

        prototype.setPropertyIsEnumerable("toString", false);
    }
}
