package flash.display {
    public final dynamic class ShaderInput {
        [Ruffle(InternalSlot)]
        private var _channels: int;

        private var _height: int;

        [Ruffle(InternalSlot)]
        private var _index: int;

        private var _object: Object;
        private var _width: int;

        public function get channels():int {
            return _channels;
        }

        public function get height():int {
            return _height;
        }

        public function set height(value:int):void {
            _height = value;
        }

        public function get index():int {
            return _index;
        }

        public function get width():int {
            return _width;
        }

        public function set width(value:int):void {
            _width = value;
        }

        public function get input():Object {
            return _object;
        }

        public function set input(value:Object):void {
            // FIXME - validate
            _object = value;
        }
    }
}
