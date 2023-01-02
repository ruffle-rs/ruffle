package {
    [Ruffle(InstanceAllocator)]
    public final dynamic class XML {
        public function XML(value:* = undefined) {
            this.init(value);
        }

        private native function init(value:*): void;

        AS3 native function localName():Object;
        AS3 native function toXMLString():String;
        
        public native function toString():String;
    }
}