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
            if (this._value) {
                return this._value.concat();
            }
            return null;
        }
        public function set value(value:Array):void {
            // FIXME - perform validation
            if (value) {
                this._value = value.concat();
            } else {
                this._value = null;
            }

        }
    }
}
