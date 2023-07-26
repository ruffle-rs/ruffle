package flash.text.engine {
    public final class TabStop {
        // FIXME: These should be getters/setters to match Flash
        public var alignment:String;
        public var position:Number;
        public var decimalAlignmentToken:String;
        
        public function TabStop(alignment:String = "start", position:Number = 0, decimalAlignmentToken:String = "") {
            this.alignment = alignment;
            this.position = position;
            this.decimalAlignmentToken = decimalAlignmentToken;
        }
    }
}
