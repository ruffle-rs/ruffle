package flash.display {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import flash.utils.ByteArray;

    public class Shader {
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
            stub_getter("flash.display.Shader", "precisionHint");
            return this._precisionHint;
        }

        public function set precisionHint(value:String):void {
            stub_setter("flash.display.Shader", "precisionHint");
            this._precisionHint = value;
        }
    }
}

