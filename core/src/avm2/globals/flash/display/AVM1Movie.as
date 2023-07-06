package flash.display {
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class AVM1Movie extends DisplayObject {
        public function AVM1Movie() {
            // Should be inaccessible
        }
        
        public function call(functionName:String, ... rest):* {
            stub_method("flash.display.AVM1Movie", "call");
            return null;
        }
        
        public function addCallback(name:String, fn:Function):void {
            stub_method("flash.display.AVM1Movie", "addCallback");
        }
    }
}
