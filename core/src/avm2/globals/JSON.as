package {
    [API("674")]
    [Ruffle(Abstract)]
    public final class JSON {
        public static function parse(text:String, reviver:Function = null):Object {
            if (text == null) {
                Error.throwError(SyntaxError, 1132);
            }

            return parseCore(text, reviver);
        }

        private static native function parseCore(text:String, reviver:Function):Object;

        public static native function stringify(value:Object, replacer:* = null, space:* = null):String;
    }
}
