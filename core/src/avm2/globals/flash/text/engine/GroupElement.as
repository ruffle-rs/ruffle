package flash.text.engine {
    import __ruffle__.stub_method;

    import flash.events.EventDispatcher;

    public final class GroupElement extends ContentElement {
        public function GroupElement(elements:Vector.<ContentElement> = null, elementFormat:ElementFormat = null, eventMirror:EventDispatcher = null, textRotation:String = "rotate0") {
            super(elementFormat, eventMirror, textRotation);
            this.setElements(elements);
        }

        public function setElements(value:Vector.<ContentElement>):void {
            stub_method("flash.text.engine.GroupElement", "setElements");
        }

        public function replaceElements(beginIndex:int, endIndex:int, newElements:Vector.<ContentElement>):Vector.<ContentElement> {
            stub_method("flash.text.engine.GroupElement", "replaceElements");
            return new Vector.<ContentElement>();
        }
    }
}

