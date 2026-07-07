package flash.text.engine {
    [API("662")]
    [Ruffle(InstanceAllocator)]
    public final class TabStop {
        public function TabStop(alignment:String = "start", position:Number = 0, decimalAlignmentToken:String = "") {
            this.alignment = alignment;
            this.position = position;
            this.decimalAlignmentToken = decimalAlignmentToken;
        }

        public native function get alignment():String;
        public native function set alignment(value:String):void;

        public native function get position():Number;
        public native function set position(value:Number):void;

        public native function get decimalAlignmentToken():String;
        public native function set decimalAlignmentToken(value:String):void;
    }
}
