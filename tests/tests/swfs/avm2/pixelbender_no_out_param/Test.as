package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "no_out.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        try {
            new Shader(new ShaderBytes());
        } catch (e) {
            trace(e.getStackTrace());
        }
    }
}
}
