package {
import flash.display.*;
import flash.text.*;
import flash.geom.*;

public class Test extends Sprite {
    public function Test() {
        trace("// Default values (stage)");
        printProps(stage);
        trace("// Default values (root)");
        printProps(root);
        trace("// Default values (object)");
        printProps(new Sprite());

        var s = new Sprite();
        trace("// Before setting");
        printProps(s);
        s.transform.perspectiveProjection = new PerspectiveProjection();
        trace("// After setting to non-null");
        printProps(s);
        s.transform.perspectiveProjection = null;
        trace("// After setting to null");
        printProps(s);

        trace("// After setting stage to null");
        stage.transform.perspectiveProjection = null;
        printProps(stage);

        trace("// After setting root to null");
        root.transform.perspectiveProjection = null;
        printProps(root);
    }

    private function printProps(o: DisplayObject): void {
        trace("  z = " + o.z);
        trace("  matrix = " + o.transform.matrix);
        trace("  matrix3D = " + o.transform.matrix3D);
        trace("  perspectiveProjection = " + o.transform.perspectiveProjection);
    }
}
}
