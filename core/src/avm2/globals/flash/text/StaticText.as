package flash.text {
    import flash.display.DisplayObject;

    [Ruffle(Abstract)]
    public final class StaticText extends DisplayObject {
        public native function get text():String;
    }
}
