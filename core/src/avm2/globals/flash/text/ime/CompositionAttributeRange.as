package flash.text.ime {
    public final class CompositionAttributeRange {
        public var converted:Boolean;

        public var relativeStart:int;
        
        public var relativeEnd:int;
        
        public var selected:Boolean;
        
        public function CompositionAttributeRange(relativeStart:int, relativeEnd:int, selected:Boolean, converted:Boolean) {
            this.relativeStart = relativeStart;
            this.relativeEnd = relativeEnd;
            this.selected = selected;
            this.converted = converted;
        }
    }
}

