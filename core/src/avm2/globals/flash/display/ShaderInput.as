package flash.display {
    public final dynamic class ShaderInput {
        [Ruffle(NativeAccessible)]
        private var _channels: int;

        [Ruffle(NativeAccessible)]
        private var _height: int;

        [Ruffle(NativeAccessible)]
        private var _index: int;

        [Ruffle(NativeAccessible)]
        private var _input: Object;

        [Ruffle(NativeAccessible)]
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
            return this._input;
        }

        public function set input(value:Object):void {
            // FIXME - validate
            this._input = value;
        }
    }
}
