package {
import flash.display.*;

public class Test extends Sprite {
    private var stage3d:Stage3D;

    public function Test() {
        stage3d = stage.stage3Ds[0];

        setX(-1.1234567891234567);
        setY(4.569999);

        trace("setting x to empty string");
        setX("");

        trace("setting x to \"2.4\"");
        setX("2.4");

        trace("setting x to \"#\"");
        setX("#");

        trace("setting y to \"#\"");
        setY("#");

        trace("setting x to null");
        setX(null);

        trace("setting y to null");
        setY(null);

        setX(8191);
        setX(8191.0001);
        setX(-8192);
        setX(-8192.0001);

        setY(8191);
        setY(8191.0001);
        setY(-8192);
        setY(-8192.0001);
    }

    private function setX(x:*) {
        try {
            stage3d.x = x;
            trace("x: " + stage3d.x + ", y: " + stage3d.y);
        } catch(e) {
            trace(e);
        }
    }

    private function setY(y:*) {
        try {
            stage3d.y = y;
            trace("x: " + stage3d.x + ", y: " + stage3d.y);
        } catch(e) {
            trace(e);
        }
    }
}
}
