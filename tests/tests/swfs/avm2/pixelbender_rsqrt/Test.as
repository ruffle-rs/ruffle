package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "rsqrt.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        testParameter(0.0);
        testParameter(1.0);
        testParameter(5.0);
        testParameter(9.0);
        testParameter(16.0);
        testParameter(256.0);
        testParameter(-1.0);
        testParameter(-16.0);
    }

    private function testParameter(value:Number) {
        trace("Input: " + value);
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);

        shaderJob.shader.data.inputValue.value = [value];

        try {
            shaderJob.start(true);
            trace("Result: " + input.getPixel32(0, 0).toString(16));
        } catch (e) {
            trace("Error while starting: " + e);
        }
        trace("=================");
    }
}
}
