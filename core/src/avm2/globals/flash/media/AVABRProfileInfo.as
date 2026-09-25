package flash.media {
    [API("688")]
    public class AVABRProfileInfo {
        private var _bitsPerSecond:int;
        private var _width:int;
        private var _height:int;

        public function AVABRProfileInfo(
            bitsPerSecond:int,
            width:int,
            height:int
        ) {
            this._bitsPerSecond = bitsPerSecond;
            this._width = width;
            this._height = height;
        }

        public function get bitsPerSecond():int {
            return this._bitsPerSecond;
        }

        public function get width():int {
            return this._width;
        }

        public function get height():int {
            return this._height;
        }
    }
}
