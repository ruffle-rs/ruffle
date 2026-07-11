package {

import flash.display.*;

public class Test extends MovieClip {
    public function Test() {
        testThrowing(function() { new GraphicsPath(null, null, "invalid"); });
        testThrowing(function() { new GraphicsPath(null, null, "evenOdd"); });
        testThrowing(function() { new GraphicsPath(null, null, "evenodd"); });
        testThrowing(function() { new GraphicsPath(null, null, "nonzero"); });
        testThrowing(function() { new GraphicsPath(null, null, null); });

        var gp:GraphicsPath = new GraphicsPath();
        trace("gp.commands=" + gp.commands);
        trace("gp.data=" + gp.data);
        trace("gp.winding=" + gp.winding);

        gp.commands = Vector.<int>([1]);
        trace("gp.commands=" + gp.commands);

        gp.data = Vector.<Number>([1]);
        trace("gp.data=" + gp.data);

        gp.winding = "evenOdd";
        trace("gp.winding=" + gp.winding);

        gp.winding = "nonZero";
        trace("gp.winding=" + gp.winding);

        testThrowing(function() { gp.winding = "invalid"; });
        testThrowing(function() { gp.winding = "evenodd"; });
        testThrowing(function() { gp.winding = "nonzero"; });
        testThrowing(function() { gp.winding = null; });
    }

    private function testThrowing(f:Function) {
        try {
            f();
            trace("Did not throw");
        } catch (e) {
            trace("Threw: " + e.getStackTrace());
        }
    }
}

}
