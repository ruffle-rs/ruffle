package flash.text.engine {
    import flash.events.EventDispatcher;

    public final class GroupElement extends ContentElement {
        public function GroupElement(elements:Vector.<ContentElement> = null, elementFormat:ElementFormat = null, eventMirror:EventDispatcher = null, textRotation:String = "rotate0") {
            super(elementFormat, eventMirror, textRotation);
            this.setElements(elements);
        }
    }
}

