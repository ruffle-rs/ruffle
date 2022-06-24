package flash.text {
    public class TextExtent {
        public var width: Number;
        public var height: Number;
        public var textFieldWidth: Number;
        public var textFieldHeight: Number;
        public var ascent: Number;
        public var descent: Number;

        public function TextExtent(width: Number, height: Number, textFieldWidth: Number, textFieldHeight: Number, ascent: Number, descent: Number) {
            this.width = width;
            this.height = height;
            this.textFieldWidth = textFieldWidth;
            this.textFieldHeight = textFieldHeight;
            this.ascent = ascent;
            this.descent = descent;
        }
    }
}
