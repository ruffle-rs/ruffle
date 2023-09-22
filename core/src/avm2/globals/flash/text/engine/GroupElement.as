package flash.text.engine {
    import __ruffle__.stub_method;

    import flash.events.EventDispatcher;

    public final class GroupElement extends ContentElement {
        internal var _elements = null;

        public function GroupElement(elements:Vector.<ContentElement> = null, elementFormat:ElementFormat = null, eventMirror:EventDispatcher = null, textRotation:String = "rotate0") {
            super(elementFormat, eventMirror, textRotation);
            this.setElements(elements);
        }

        public function get elementCount():int {
            return this._elements.length;
        }

        public function setElements(elements:Vector.<ContentElement>):void {
            if (elements == null) {
                this._elements = new Vector.<ContentElement>();
            } else {
                this._elements = elements.AS3::concat();
            }
        }

        public function replaceElements(beginIndex:int, endIndex:int, newElements:Vector.<ContentElement>):Vector.<ContentElement> {
            if (beginIndex == endIndex && newElements.length == 1) {
                if (beginIndex == this._elements.length) {
                    this._elements.push(newElements[0]);
                } else if (beginIndex < this._elements.length) {
                    this._elements[beginIndex] = newElements[0];
                } else {
                    throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
                }
            } else {
                stub_method("flash.text.engine.GroupElement", "replaceElements", "with beginIndex != endIndex or newElements.length != 1");
                return new Vector.<ContentElement>();
            }
        }

        // FIXME: This is wrong, FP doesn't do an override of `get text` in GroupElement
        override public function get text():String {
            var resultingText:String = "";

            for (var i = 0; i < this._elements.length; i ++) {
                resultingText += this._elements[i].text;
            }

            return resultingText;
        }
    }
}

