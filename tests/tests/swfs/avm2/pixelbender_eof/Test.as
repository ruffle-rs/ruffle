package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "eof.pbj", mimeType="application/octet-stream")]
    public static var ShaderEof: Class;

    public function Test() {
        try {
            new Shader(new ShaderEof());
            trace("ShaderEof compiled");
        } catch (e) {
            trace("Error compiling ShaderEof");
            trace(e.getStackTrace());
        }
    }
}
}
