package flash.display {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import __ruffle__.stub_constructor;
    import flash.events.EventDispatcher;

    public class ShaderJob extends EventDispatcher {

        private var _shader:Shader;
        private var _target:Object;
        
        public function ShaderJob(shader:Shader = null, target:Object = null, width:int = 0, height:int = 0) {
            this._shader = shader;
            this._target = target;
            stub_constructor("flash.display.ShaderJob");
        }

        public function cancel():void {
            stub_method("flash.display.ShaderJob", "cancel")
        }

        public native function start(waitForCompletion:Boolean = false):void;

        public function get height():int {
            stub_getter("flash.display.ShaderJob", "height");
            return 0;
        }

        public function set height(value:int):void {
            stub_setter("flash.display.ShaderJob", "height");
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