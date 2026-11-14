package flash.display {
    import flash.events.EventDispatcher;
    import flash.events.ShaderEvent;
    import flash.utils.ByteArray;

    [API("662")]
    public class ShaderJob extends EventDispatcher {
        [Ruffle(NativeAccessible)]
        private var _shader:Shader;

        [Ruffle(NativeAccessible)]
        private var _target:Object;

        [Ruffle(NativeAccessible)]
        private var _width:int;

        [Ruffle(NativeAccessible)]
        private var _height:int;

        private var _progress:Number = 0;

        public function ShaderJob(shader:Shader = null, target:Object = null, width:int = 0, height:int = 0) {
            this._shader = shader;
            this._target = target;
            this._width = width;
            this._height = height;
        }

        public function cancel():void {
            this._progress = 0;
        }

        public function start(waitForCompletion:Boolean = false):void {
            this._progress = 0;
            this.startInternal(waitForCompletion);
            this._progress = 1;
            if (!waitForCompletion) {
                this.dispatchEvent(new ShaderEvent(
                    ShaderEvent.COMPLETE,
                    false,
                    false,
                    this._target as BitmapData,
                    this._target as ByteArray,
                    this._target as Vector.<Number>
                ));
            }
        }

        private native function startInternal(waitForCompletion:Boolean = false):void;

        public function get height():int {
            return this._height;
        }

        public function set height(value:int):void {
            this._height = value;
        }

        public function get width():int {
            return this._width;
        }

        public function set width(value:int):void {
            this._width = value;
        }

        public function get progress():Number {
            return this._progress;
        }

        public function get shader():Shader {
            return this._shader;
        }

        public function set shader(value:Shader):void {
            this._shader = value;
        }

        public function get target():Object {
            return this._target;
        }

        public function set target(value:Object):void {
            this._target = value;
        }
    }
}
