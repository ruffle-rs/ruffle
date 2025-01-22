package flash.display {
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
            return this._value.concat();
        }
        public function set value(value:Array):void {
            // FIXME - perform validation
            this._value = value.concat();
        }
    }
}
