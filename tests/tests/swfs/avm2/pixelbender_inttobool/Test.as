package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "inttobool.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        // Test various integer values to see IntToBool behavior
        testParameter(-100);
        testParameter(-5);
        testParameter(-1);
        testParameter(0);
        testParameter(1);
        testParameter(2);
        testParameter(5);
        testParameter(100);
        testParameter(255);
        testParameter(256);
        testParameter(1000);
        testParameter(-1000);
    }

    private function testParameter(value:int) {
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