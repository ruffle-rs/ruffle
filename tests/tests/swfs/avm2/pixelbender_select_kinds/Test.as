package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "select_fff.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect1: Class;
    [Embed(source = "select_ffi.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect2: Class;
    [Embed(source = "select_fif.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect3: Class;
    [Embed(source = "select_iff.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect4: Class;
    [Embed(source = "select_fii.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect5: Class;
    [Embed(source = "select_ifi.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect6: Class;
    [Embed(source = "select_iif.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect7: Class;
    [Embed(source = "select_iii.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect8: Class;

    public function Test() {
        testShaderCompilation(1, new ShaderSelect1());
        testShaderCompilation(2, new ShaderSelect2());
        testShaderCompilation(3, new ShaderSelect3());
        testShaderCompilation(4, new ShaderSelect4());
        testShaderCompilation(5, new ShaderSelect5());
        testShaderCompilation(6, new ShaderSelect6());
        testShaderCompilation(7, new ShaderSelect7());
        testShaderCompilation(8, new ShaderSelect8());
    }

    private function testShaderCompilation(i:int, bytes:*) {
        try {
            new Shader(bytes);
            trace("Shader " + i + " compiled");
        } catch (e) {
            trace("Error compiling shader " + i + ": " + e);
        }
    }
}
}
