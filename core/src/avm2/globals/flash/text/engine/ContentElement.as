package flash.text.engine {
    import flash.events.EventDispatcher;

    [API("662")]
    [Ruffle(InstanceAllocator)]
    public class ContentElement {
        public static const GRAPHIC_ELEMENT:uint = 65007;

        public var userData;

        public function ContentElement(
            elementFormat:ElementFormat = null,
            eventMirror:EventDispatcher = null,
            textRotation:String = "rotate0"
        ) {
            // FIXME: `new ContentElement()` throws an error in Flash; see TextJustifier
            this.elementFormat = elementFormat;
        }

        [Ruffle(NativeCallable)]
        public native function get text():String;

        public function get rawText():String {
            return this.text;
        }

        public native function get elementFormat():ElementFormat;
        public native function set elementFormat(value:ElementFormat):void;
    }
}
