package flash.text {
    [Ruffle(InstanceAllocator)]
    public class Font {
        public static native function enumerateFonts(enumerateDeviceFonts:Boolean = false):Array;
        public static native function registerFont(font:Class):void;

        public native function get fontName():String;
        public native function get fontStyle():String;
        public native function get fontType():String;

        public native function hasGlyphs(str:String):Boolean;
    }
}