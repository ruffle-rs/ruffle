package {
    [Ruffle(InstanceAllocator)]
    public final dynamic class XMLList {

        public function XMLList(value:* = undefined) {
            this.init(value);
        }

        private native function init(value:*): void;

        AS3 native function hasSimpleContent():Boolean;
        AS3 native function length():int
        AS3 native function children():XMLList;

        public native function toString():String;
    }
}