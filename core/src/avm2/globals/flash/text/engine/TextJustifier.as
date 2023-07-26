package flash.text.engine {

    [Ruffle(NativeInstanceInit)]
    public class TextJustifier {
        public function TextJustifier(locale:String, lineJustification:String) {
            throw new ArgumentError("Error #2012: TextJustifier$ class cannot be instantiated.", 2012);
        }
        
        public function clone():TextJustifier {
            return null;
        }
    }
}
