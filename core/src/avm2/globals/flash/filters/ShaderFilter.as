package flash.filters {
    import flash.display.Shader;

    public class ShaderFilter extends BitmapFilter {
        [Ruffle(NativeAccessible)]
        private var _bottomExtension:int = 0;

        [Ruffle(NativeAccessible)]
        private var _leftExtension:int = 0;

        [Ruffle(NativeAccessible)]
        private var _rightExtension:int = 0;

        [Ruffle(NativeAccessible)]
        private var _topExtension:int = 0;

        [Ruffle(NativeAccessible)]
        private var _shader;

        public function ShaderFilter(shader:Shader = null) {
            this._shader = shader;
        }

        public function get bottomExtension():int {
            return this._bottomExtension;
        }

        public function set bottomExtension(value:int):void {
            this._bottomExtension = value;
        }

        public function get leftExtension():int {
            return this._leftExtension;
        }

        public function set leftExtension(value:int):void {
            this._leftExtension = value;
        }

        public function get rightExtension():int {
            return this._rightExtension;
        }

        public function set rightExtension(value:int):void {
            this._rightExtension = value;
        }

        public function get topExtension():int {
            return this._topExtension;
        }

        public function set topExtension(value:int):void {
            this._topExtension = value;
        }

        public function get shader():Shader {
            return this._shader;
        }

        public function set shader(value:Shader):void {
            this._shader = value;
        }

        // ShaderFilter is the only filter class that doesn't override clone
    }
}
