package flash.display {
    import flash.utils.ByteArray;

    [API("662")]
    public class Shader {
        [Ruffle(NativeAccessible)]
        private var _data:ShaderData;

        private var _precisionHint:String = ShaderPrecision.FULL;

        public function Shader(bytecode:ByteArray = null) {
            if (bytecode) {
                this.byteCode = bytecode;
            }
        }

        public function set byteCode(code:ByteArray):void {
            this._data = new ShaderData(code);
        }

        public function get data():ShaderData {
            return this._data;
        }

        public function set data(value:ShaderData):void {
            this._data = value;
        }

        public function get precisionHint():String {
            return this._precisionHint;
        }

        public function set precisionHint(value:String):void {
            if (value != ShaderPrecision.FULL && value != ShaderPrecision.FAST) {
                Error.throwError(ArgumentError, 2008, "precisionHint");
            }
            this._precisionHint = value;
        }
    }
}
