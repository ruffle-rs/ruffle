package flash.events {
    import flash.utils.ByteArray;

    // FIXME: Check if the class is versioned.
    public class DatagramSocketDataEvent extends Event {
        public static const DATA = "data";

        public var data:ByteArray;
        public var dstAddress:String;
        public var dstPort:int;
        public var srcAddress:String;
        public var srcPort:int;

        public function DatagramSocketDataEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, srcAddress:String = "", srcPort:int = 0, dstAddress:String = "", dstPort:int = 0, data:ByteArray = null) {
            super(type,bubbles,cancelable);
            this.data = data;
            this.dstAddress = dstAddress;
            this.dstPort = dstPort;
            this.srcAddress = srcAddress;
            this.srcPort = srcPort;
        }

        override public function clone() : Event {
            return new DatagramSocketDataEvent(this.type,this.bubbles,this.cancelable,this.srcAddress,this.srcPort,this.dstAddress,this.dstPort,this.data);
        }

        override public function toString() : String {
            // FIXME: Verify the order.
            return this.formatToString("DatagramSocketDataEvent","type","bubbles","cancelable","eventPhase","srcAddress","srcPort","dstAddress","dstPort","data");
        }
    }
}