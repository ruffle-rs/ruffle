package flash.display {
    import __ruffle__.stub_method;

    [Ruffle(Abstract)]
    public class AVM1Movie extends DisplayObject {
        // TODO: Can these two methods do anything else besides
        // throw an error? Should we keep the stubs?

        public function call(functionName:String, ...rest):* {
            stub_method("flash.display.AVM1Movie", "call");
            Error.throwError(Error, 2014);
            return null;
        }

        public function addCallback(name:String, fn:Function):void {
            stub_method("flash.display.AVM1Movie", "addCallback");
            Error.throwError(Error, 2014);
        }
    }
}
