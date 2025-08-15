package flash.text.engine {
    import flash.events.EventDispatcher;

    [API("662")]
    public class ContentElement {
        public static const GRAPHIC_ELEMENT:uint = 65007;
        public var userData;

        internal var _text:String = null;

        [Ruffle(NativeAccessible)]
        private var _elementFormat:ElementFormat;

        public function ContentElement(elementFormat:ElementFormat = null, eventMirror:EventDispatcher = null, textRotation:String = "rotate0") {
            // FIXME: `new ContentElement()` throws an error in Flash; see TextJustifier
            this._elementFormat = elementFormat;
        }

        [Ruffle(NativeCallable)]
        public function get text():String {
            return this._text;
        }

        public function get rawText():String {
            return this._text;
        }

        public function get elementFormat():ElementFormat {
            return this._elementFormat;
        }

        public function set elementFormat(value:ElementFormat):void {
            this._elementFormat = value;
        }
    }
}
