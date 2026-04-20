package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "output_float.pbj", mimeType="application/octet-stream")]
    public static var ShaderFloat: Class;
    [Embed(source = "output_float2.pbj", mimeType="application/octet-stream")]
    public static var ShaderFloat2: Class;
    [Embed(source = "output_float3.pbj", mimeType="application/octet-stream")]
    public static var ShaderFloat3: Class;
    [Embed(source = "output_float4.pbj", mimeType="application/octet-stream")]
    public static var ShaderFloat4: Class;

    public function Test() {
        testShader(new ShaderFloat());
        testShader(new ShaderFloat2());
        testShader(new ShaderFloat3());
        testShader(new ShaderFloat4());
    }

    private function testShader(shaderBytes:*) {
        var input = new BitmapData(1, 1);
        try {
            var shaderJob = new ShaderJob(new Shader(shaderBytes), input);
        } catch (e) {
            trace("Error while creating: " + e);
            return;
        }

        try {
            shaderJob.start(true);
            trace("Result: " + input.getPixel(0, 0).toString(16));
            trace("Result: " + input.getPixel32(0, 0).toString(16));
        } catch (e) {
            trace("Error while starting: " + e);
        }

        trace("=================");
    }
}
}
