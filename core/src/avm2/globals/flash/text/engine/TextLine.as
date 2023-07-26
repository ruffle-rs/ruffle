package flash.text.engine {
    import flash.display.DisplayObjectContainer;
    
    [Ruffle(NativeInstanceInit)]
    public final class TextLine extends DisplayObjectContainer {
        public static const MAX_LINE_WIDTH:int = 1000000;
        
        public var userData;
        
        public function TextLine() {
            throw new ArgumentError("Error #2012: TextLine$ class cannot be instantiated.", 2012);
        }
    }
}
