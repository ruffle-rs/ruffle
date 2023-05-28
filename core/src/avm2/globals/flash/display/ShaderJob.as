package flash.display {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import __ruffle__.stub_constructor;
    import flash.events.EventDispatcher;

    public class ShaderJob extends EventDispatcher {
        
        public function ShaderJob(shader:Shader = null, target:Object = null, width:int = 0, height:int = 0) {
            stub_constructor("flash.display.ShaderJob");
        }

        public function cancel():void {
            stub_method("flash.display.ShaderJob", "cancel")
        }

        public function start(waitForCompletion:Boolean = false):void {
            stub_method("flash.display.ShaderJob", "start")
        }

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
            stub_getter("flash.display.ShaderJob", "shader");
            return null;
        }

        public function set shader(value:Shader):void {
            stub_setter("flash.display.ShaderJob", "shader");
        }
    }
}