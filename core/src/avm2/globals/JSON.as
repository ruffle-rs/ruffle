package {
    public final class JSON {
        public static native function parse(text:String, reviver:Function = null): Object;
        public static native function stringify(value:Object, replacer:* = null, space:* = null): String;
    }
}
