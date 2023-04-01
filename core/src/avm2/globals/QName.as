package {
    [Ruffle(InstanceAllocator)]
    public final class QName {
        public function QName(uri:* = undefined, localName:*=undefined) {
            this.init(uri, localName)
        }

        private native function init(uri:*, localName:*):void;

        public native function get localName():String;
        public native function get uri():String;

        AS3 native function toString():String;
        AS3 function valueOf():QName {
            return this;
        }

        prototype.toString = function():String {
            var self:QName = this;
            return self.AS3::toString();
        }
        prototype.valueOf = function():QName {
            var self:QName = this;
            return self.AS3::valueOf();
        }

        prototype.setPropertyIsEnumerable("toString", false);
        prototype.setPropertyIsEnumerable("valueOf", false);
    }
}