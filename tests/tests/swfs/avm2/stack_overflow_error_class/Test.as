package {
import flash.display.Sprite;
import flash.errors.StackOverflowError;

public class Test extends Sprite {
    public function Test() {
        describe("new StackOverflowError()", new StackOverflowError());
        describe("new StackOverflowError(\"msg\")", new StackOverflowError("msg"));
        describe("new StackOverflowError(\"msg\", 5)", new StackOverflowError("msg", 5));

        trace("// prototype");
        trace("has own name: " + StackOverflowError.prototype.hasOwnProperty("name"));
        trace("name: " + StackOverflowError.prototype.name);
        trace("is Error: " + (new StackOverflowError() is Error));

        trace("// throw and catch");
        try {
            throw new StackOverflowError("thrown", 7);
        } catch (e:*) {
            trace(Object(e).constructor);
            trace(Object(e).getStackTrace());
        }
    }

    private function describe(name:String, e:StackOverflowError):void {
        trace("// " + name);
        trace("name: " + e.name);
        trace("message: " + e.message);
        trace("errorID: " + e.errorID);
        trace("toString: " + e.toString());
        trace("stack: " + e.getStackTrace());
    }
}
}
