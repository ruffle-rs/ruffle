package flash.text {
    import flash.display.DisplayObject;
    
    [Ruffle(NativeInstanceInit)]
    public class StaticText extends DisplayObject {
        public function StaticText() {
            throw new ArgumentError("Error #2012: StaticText$ class cannot be instantiated.", 2012);
        }
        public native function get text():String;
    }
}