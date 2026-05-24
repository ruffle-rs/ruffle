package flash.text.engine {
    import flash.events.EventDispatcher;

    [API("662")]
    public final class GroupElement extends ContentElement {
        internal var _elements = null;

        public function GroupElement(
            elements:Vector.<ContentElement> = null,
            elementFormat:ElementFormat = null,
            eventMirror:EventDispatcher = null,
            textRotation:String = "rotate0"
        ) {
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

        public function replaceElements(
            beginIndex:int,
            endIndex:int,
            newElements:Vector.<ContentElement>
        ):Vector.<ContentElement> {
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

        public function getElementAtCharIndex(charIndex:int):ContentElement {
            var pos:int = 0;
            for (var i:int = 0; i < this._elements.length; i++) {
                var elem:ContentElement = this._elements[i];
                var len:int = elem.text != null ? elem.text.length : 0;
                if (charIndex >= pos && charIndex < pos + len) {
                    return elem;
                }
                pos += len;
            }
            return null;
        }

        public function groupElements(beginIndex:int, endIndex:int):GroupElement {
            if (beginIndex < 0 || endIndex < beginIndex || endIndex > this._elements.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }

            var taken:Vector.<ContentElement> = this._elements.AS3::splice(beginIndex, endIndex - beginIndex);
            var grouped:GroupElement = new GroupElement(taken, this.elementFormat);
            this._elements.AS3::insertAt(beginIndex, grouped);
            return grouped;
        }

        public function ungroupElements(groupIndex:int):void {
            if (groupIndex < 0 || groupIndex >= this._elements.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }

            var elem:ContentElement = this._elements[groupIndex];
            if (!(elem is GroupElement)) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }

            var inner:Vector.<ContentElement> = new Vector.<ContentElement>();
            var grp:GroupElement = elem as GroupElement;
            for (var i:int = 0; i < grp.elementCount; i++) {
                inner.push(grp.getElementAt(i));
            }

            this._elements.AS3::splice(groupIndex, 1);
            for (var j:int = 0; j < inner.length; j++) {
                this._elements.AS3::insertAt(groupIndex + j, inner[j]);
            }
        }

        public function mergeTextElements(beginIndex:int, endIndex:int):TextElement {
            if (beginIndex < 0 || endIndex < beginIndex || endIndex > this._elements.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }

            var merged:String = "";
            for (var i:int = beginIndex; i < endIndex; i++) {
                var e:ContentElement = this._elements[i];
                if (!(e is TextElement)) {
                    throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
                }
                merged += (e as TextElement).text;
            }

            var first:TextElement = this._elements[beginIndex] as TextElement;
            first.text = merged;
            this._elements.AS3::splice(beginIndex + 1, endIndex - beginIndex - 1);
            return first;
        }

        public function splitTextElement(elementIndex:int, splitIndex:int):TextElement {
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
