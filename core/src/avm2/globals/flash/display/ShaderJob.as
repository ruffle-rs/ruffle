package flash.display {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_constructor;
    import flash.events.EventDispatcher;

    public class ShaderJob extends EventDispatcher {

        private var _shader:Shader;
        private var _target:Object;
        private var _width:int;
        private var _height:int;
        
        public function ShaderJob(shader:Shader = null, target:Object = null, width:int = 0, height:int = 0) {
            this._shader = shader;
            this._target = target;
            this._width = width;
            this._height = height;
            stub_constructor("flash.display.ShaderJob");
        }

        public function cancel():void {
            stub_method("flash.display.ShaderJob", "cancel")
        }

        public native function start(waitForCompletion:Boolean = false):void;

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
            stub_getter("flash.display.ShaderJob", "progress")
            return 0;
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
