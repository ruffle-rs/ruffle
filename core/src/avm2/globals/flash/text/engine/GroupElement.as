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

        public function getElementAt(index:int):ContentElement {
            if (index < 0 || index >= this._elements.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            return this._elements[index];
        }

        public function getElementIndex(element:ContentElement):int {
            return this._elements.indexOf(element);
         }

        public function setElements(elements:Vector.<ContentElement>):void {
            if (elements == null) {
                this._elements = new Vector.<ContentElement>();
            } else {
                this._elements = elements.AS3::concat();
            }
        }

        public function replaceElements(beginIndex:int, endIndex:int, newElements:Vector.<ContentElement>):Vector.<ContentElement> {
            // This some sort of special case that doesn't throw.
            if (beginIndex == endIndex && newElements == null) {
                return null;
            }
            if (beginIndex < 0 || beginIndex > this._elements.length ||
                endIndex < 0 || endIndex > this._elements.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }

            var old = this._elements.AS3::splice(beginIndex, endIndex - beginIndex);
            if (newElements) {
                for (var i = 0; i < newElements.length; i++) {
                    this._elements.AS3::insertAt(beginIndex + i, newElements[i]);
                }
            }
            return old;
        }

        public function splitTextElement(elementIndex:int, splitIndex:int): TextElement {
            var element = getElementAt(elementIndex);
            if (!(element instanceof TextElement)) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }

            var text = element.text;
            if (splitIndex < 0 || splitIndex >= text.length) {
                 throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }

            element.text = text.slice(0, splitIndex);
            var newTextElement = new TextElement(text.slice(splitIndex));
            this._elements.AS3::insertAt(elementIndex + 1, newTextElement);
            return newTextElement;
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

