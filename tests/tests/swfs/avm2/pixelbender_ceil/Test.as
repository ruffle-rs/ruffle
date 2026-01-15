package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "ceil_ii.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesII: Class;
    [Embed(source = "ceil_fi.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesFI: Class;
    [Embed(source = "ceil_if.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesIF: Class;
    [Embed(source = "ceil_ff.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesFF: Class;
    [Embed(source = "ceil.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;
    [Embed(source = "ceil4.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes4: Class;

    public function Test() {
        trace("ii");
        testLoadingShader(new ShaderBytesII());
        trace("fi");
        testLoadingShader(new ShaderBytesFI());
        trace("if");
        testLoadingShader(new ShaderBytesIF());
        trace("ff");
        testLoadingShader(new ShaderBytesFF());

        testFloat(0.0);
        testFloat(0.1);
        testFloat(0.4);
        testFloat(0.5);
        testFloat(0.6);
        testFloat(0.9);
        testFloat(1.0);
        testFloat(1.1);
        testFloat(1.4);
        testFloat(1.5);
        testFloat(1.6);
        testFloat(1.9);
        testFloat(2.0);
        testFloat(-0.0);
        testFloat(-0.1);
        testFloat(-0.4);
        testFloat(-0.5);
        testFloat(-0.6);
        testFloat(-0.9);
        testFloat(-1.0);
        testFloat(-1.1);
        testFloat(-1.4);
        testFloat(-1.5);
        testFloat(-1.6);
        testFloat(-1.9);
        testFloat(-2.0);
    }

    private function testLoadingShader(shaderBytes:*) {
        var input = new BitmapData(1, 1);
        try {
            var shaderJob = new ShaderJob(new Shader(shaderBytes), input);
            shaderJob.start(true);
        } catch (e) {
            trace("Error: " + e.getStackTrace());
        }
    }

    private function testFloat(value:Number) {
        testFloat0(value, new ShaderBytes());
        testFloat0(value, new ShaderBytes4());
    }

    private function testFloat0(value:Number, shaderBytes: *) {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(shaderBytes), input);
        shaderJob.shader.data.floatInput.value = [value];
        shaderJob.start(true);
        trace(value + " -> " + input.getPixel32(0, 0).toString(16));
    }
}
}
