package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public final class QName {
        public static const length = 2;
        
        public function QName(uri:* = undefined, localName:* = undefined) {
            this.init(arguments);
        }

        private native function init(args:Array):void;

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

        prototype.setPropertyIsEnumerable("toString", false);
    }
}
