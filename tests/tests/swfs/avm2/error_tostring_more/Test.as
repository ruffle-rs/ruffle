package {
    import flash.display.Sprite;

    import flash.errors.IOError;

    import flash.errors.DRMManagerError;
    import flash.errors.EOFError;
    import flash.errors.InvalidSWFError;
    import flash.errors.MemoryError;
    import flash.errors.ScriptTimeoutError;
    import flash.errors.StackOverflowError;

    public class Test extends Sprite {
        public function Test() {
            var errorClasses:Array = [DefinitionError, EOFError, EvalError, IOError, InvalidSWFError, MemoryError, ScriptTimeoutError, StackOverflowError, URIError, VerifyError];

            for (var i:int = 0; i < errorClasses.length; i ++) {
                testErrorClass(errorClasses[i]);
            }

            // DRMManagerError takes 3 params, so test it separately
            trace("Class: " + DRMManagerError);
            trace("cls.prototype.name = " + DRMManagerError.prototype.name);
            var newErrorDRMManager:Error = new DRMManagerError("My Error", 42, 10);
            trace(newErrorDRMManager.toString());
            trace(newErrorDRMManager.name);
            trace(newErrorDRMManager.errorID);
            trace(newErrorDRMManager.subErrorID);
        }

        function testErrorClass(TestedErrorClass:Class) {
            trace("Class: " + TestedErrorClass);
            trace("cls.prototype.name = " + TestedErrorClass.prototype.name);
            var newError:Error = new TestedErrorClass("My Error", 42);
            trace(newError.toString());
            trace(newError.name);
            trace(newError.errorID);
            trace();
            trace();
            trace();
        }
    }
}
