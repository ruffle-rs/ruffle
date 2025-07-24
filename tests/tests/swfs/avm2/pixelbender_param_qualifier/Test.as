package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "param_qualifier.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;
    [Embed(source = "param_qualifier2.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes2: Class;

    public function Test() {
        var i = 0;
        while (i < 256) {
            var bytes = new ShaderBytes();

            // Change the value at 0x18, if the shader fails to compile, the param
            // has qualifier "in" (because FP fails on missing out param).
            bytes.position = 0x18;
            bytes.writeByte(i);
            testShader(bytes, i);

            i += 1;
        }

        var i = 0;
        while (i < 256) {
            var bytes = new ShaderBytes2();

            // Also, make sure we can use those input parameters.
            bytes.position = 0x23;
            bytes.writeByte(i);
            testShader2(bytes, i);

            i += 1;
        }
    }

    private function testShader(bytes:*, i:Number) {
        try {
            new Shader(bytes);
            trace(i + ": out");
        } catch (e) {
            trace(i + ": in (" + e + ")");
        }
    }

    private function testShader2(bytes:*, i:Number) {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(bytes), input);

        try {
            shaderJob.shader.data.src.value = [1];
            trace(i + ": passed");
        } catch (e) {
            trace(i + ": failed");
        }
    }
}
}
