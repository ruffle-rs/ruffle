// NOTICE: Because the stubs of AVM1Movie.addCallback and/or AVM1Movie.call are unable to do anything else besides throw an error, we need to instead call the stub of LocalConnection. And anyway, the error(s) of #2014 in Ruffle is because direct interoperability between AVM2 and AVM1 via AVM1Movie.call() and/or AVM1Movie.addCallback() was never supported in the original Adobe Flash Player. Again, authors MUST use the stub of LocalConnection instead!
package flash.display {
    [Ruffle(Abstract)]
    public class AVM1Movie extends DisplayObject {
        public function call(functionName:String, ...rest):* {
            if (functionName == null) {
                Error.throwError(TypeError, 2007, "functionName");
            }
            Error.throwError(Error, 2014);
            return null;
        }

        public function addCallback(name:String, fn:Function):void {
            if (name == null) {
                Error.throwError(TypeError, 2007, "name");
            }
            if (fn == null) {
                Error.throwError(TypeError, 2007, "fn");
            }
            Error.throwError(Error, 2014);
        }
    }
}
