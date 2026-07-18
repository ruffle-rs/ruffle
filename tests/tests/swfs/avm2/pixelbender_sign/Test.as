package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "sign_ii.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesII: Class;
    [Embed(source = "sign_fi.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesFI: Class;
    [Embed(source = "sign_if.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesIF: Class;
    [Embed(source = "sign_ff.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesFF: Class;
    [Embed(source = "sign.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;
    [Embed(source = "sign4.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes4: Class;
    [Embed(source = "sign_inplace.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesInplace: Class;
    [Embed(source = "sign_multi.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytesMulti: Class;

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
        testFloat(-0.0);
        testFloat(0.1);
        testFloat(1.0);
        testFloat(100.0);
        testFloat(-0.1);
        testFloat(-1.0);
        testFloat(-100.0);
        testFloat(Number.POSITIVE_INFINITY);
        testFloat(Number.NEGATIVE_INFINITY);
        testFloat(Number.NaN);

        trace("multi");
        var input:BitmapData = new BitmapData(1, 1);
        var shaderJob:ShaderJob = new ShaderJob(new Shader(new ShaderBytesMulti()), input);
        shaderJob.start(true);
        trace(input.getPixel32(0, 0).toString(16));
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
        testFloat0(value, new ShaderBytesInplace());
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
