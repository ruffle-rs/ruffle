package flash.text.engine {
    import flash.events.EventDispatcher;
    import flash.utils.getQualifiedClassName;

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
            if (getQualifiedClassName(this) === "flash.text.engine::ContentElement") {
                throw new ArgumentError("Error #2012: ContentElement class cannot be instantiated.", 2012);
            }

            this.elementFormat = elementFormat;
        }

        [Ruffle(NativeCallable)]
        public native function get text():String;
        public native function get rawText():String;
        public native function get textBlock():TextBlock;
        public native function get textBlockBeginIndex():int;
        public native function get groupElement():GroupElement;

        [Ruffle(NativeCallable)]
        public native function get elementFormat():ElementFormat;
        public native function set elementFormat(value:ElementFormat):void;

        public native function get eventMirror():EventDispatcher;
        public native function set eventMirror(eventMirror:EventDispatcher):void;

        public native function get textRotation():String;
        public native function set textRotation(textRotation:String):void;
    }
}
