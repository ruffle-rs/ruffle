package flash.text.engine {
    import flash.events.EventDispatcher;
    import __ruffle__.stub_setter;
    import __ruffle__.stub_method;
   
    public final class TextElement extends ContentElement {
        public function TextElement(text:String = null, elementFormat:ElementFormat = null, eventMirror:EventDispatcher = null, textRotation:String = "rotate0") {
            super(elementFormat, eventMirror, textRotation);
            this.text = text;
        }
        
        // Contrary to the documentation, TextElement does not implement a getter here. It inherits the getter from ContentElement.
        public function set text(value:String):void {
            this._text = value;
        }

        public function replaceText(beginIndex:int, endIndex:int, newText:String):void {
            var realText:String = this.text;
            if (realText == null) {
                realText = "";
            }

            if (beginIndex < 0 || endIndex < 0 || beginIndex > realText.length || endIndex > realText.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }

            this.text = realText.slice(0, beginIndex) + newText + realText.slice(endIndex, realText.length);
        }
    }
}

