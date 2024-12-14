package flash.display {
    public final dynamic class ShaderParameter {
        [Ruffle(InternalSlot)]
        private var _index:int;

        [Ruffle(InternalSlot)]
        private var _type:String;

        [Ruffle(InternalSlot)]
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
