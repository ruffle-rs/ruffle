package flash.text.engine {
    import flash.events.EventDispatcher;

    [API("662")]
    public final class GroupElement extends ContentElement {
        public function GroupElement(
            elements:Vector.<ContentElement> = null,
            elementFormat:ElementFormat = null,
            eventMirror:EventDispatcher = null,
            textRotation:String = "rotate0"
        ) {
            super(elementFormat, eventMirror, textRotation);

            this.init();
            this.setElements(elements);
        }

        private native function init():void;

        public native function get elementCount():int;

        public native function getElementAt(index:int):ContentElement;

        public function getElementIndex(testElement:ContentElement):int {
            for (var i:int = 0; i < this.elementCount; i ++) {
                var element:ContentElement = this.getElementAt(i);
                if (testElement == element) {
                    return i;
                }
            }

            return -1;
        }

        public native function setElements(elements:Vector.<ContentElement>):void;

        public native function replaceElements(
            beginIndex:int,
            endIndex:int,
            newElements:Vector.<ContentElement>
        ):Vector.<ContentElement>;

        // TODO: This method should probably be implemented in native code
        public function splitTextElement(elementIndex:int, splitIndex:int):TextElement {
            var element:ContentElement = this.getElementAt(elementIndex);
            if (!(element instanceof TextElement)) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }

            var text:String = element.text;
            if (splitIndex < 0 || splitIndex >= text.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            element.text = text.slice(0, splitIndex);

            var newTextElement:ContentElement = new TextElement(text.slice(splitIndex));
            this.replaceElements(
                elementIndex + 1,
                elementIndex + 1,
                Vector.<ContentElement>([newTextElement])
            );
            return newTextElement;
        }
    }
}
