package flash.display {
    public final dynamic class ShaderInput {
        internal var _channels: int;
        internal var _height: int;
        internal var _index: int;
        internal var _object: Object;
        internal var _width: int;

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