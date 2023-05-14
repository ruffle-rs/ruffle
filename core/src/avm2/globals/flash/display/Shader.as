package flash.display {
    import flash.utils.ByteArray;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    public class Shader {
        private var _data:ShaderData;
        private var _precisionHint:String = ShaderPrecision.FULL;

        public function Shader(bytecode:ByteArray = null) {
            if (bytecode) {
                this.byteCode = bytecode;
            }
        }

        public function set byteCode(code:ByteArray):void {
            stub_setter("flash.display.Shader", "byteCode");
            this._data = new ShaderData(code);
        }

        public function get data():ShaderData {
            stub_getter("flash.display.Shader", "data");
            return this._data;
        }

        public function set data(value:ShaderData):void {
            stub_setter("flash.display.Shader", "data");
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

