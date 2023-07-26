package flash.text.engine {
    import flash.events.EventDispatcher;
    import __ruffle__.stub_setter;
   
    public final class TextElement extends ContentElement {
        public function TextElement(text:String = null, elementFormat:ElementFormat = null, eventMirror:EventDispatcher = null, textRotation:String = "rotate0") {
            super(elementFormat, eventMirror, textRotation);
        }
        
        // Contrary to the documentation, TextElement does not implement a getter here. It inherits the getter from ContentElement.
        public function set text(value:String):void {
            stub_setter("flash.text.engine.TextElement", "text");
        }
    }
}

