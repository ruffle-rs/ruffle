package flash.display {
    [API("662")]
    public final dynamic class ShaderParameter {
        [Ruffle(NativeAccessible)]
        private var _index:int;

        [Ruffle(NativeAccessible)]
        private var _type:String;

        [Ruffle(NativeAccessible)]
        private var _value:Array;

        public function get index():int {
            return this._index;
        }
        public function get type():String {
            return this._type;
        }
        public function get value():Array {
            return this._value;
        }
        public function set value(value:Array):void {
            // FIXME - Is there some validation here?
            this._value = value;
        }
    }
}
