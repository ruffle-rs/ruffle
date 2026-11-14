package flash.net {
    [Ruffle(Abstract)]
    public final class ObjectEncoding {
        public static const AMF0:uint = 0;
        public static const AMF3:uint = 3;
        public static const DEFAULT:uint = 3;

        private static var _dynamicPropertyWriter:IDynamicPropertyWriter = null;

        public static function get dynamicPropertyWriter():IDynamicPropertyWriter {
            return _dynamicPropertyWriter;
        }

        public static function set dynamicPropertyWriter(value:IDynamicPropertyWriter):void {
            _dynamicPropertyWriter = value;
        }
    }
}
