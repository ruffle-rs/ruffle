package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "div.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        testParameter(4.0, 4.0);
        testParameter(2.0, 8.0);
        testParameter(2.0, 16.0);
        testParameter(2.0, 8.0);
        testParameter(8.0, 1.0);
        testParameter(8.0, 2.0);
        testParameter(-8.0, -2.0);
        testParameter(-2.0, -8.0);
        testParameter(0.0, 2.0);
        testParameter(2.5, 8.0);
        testParameter(2.1, 8.0);
        testParameter(2.9, 8.0);
    }

    private function testParameter(left:Number, right:Number) {
        trace("Input: " + left + ", " + right);
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);

        shaderJob.shader.data.inputLeft.value = [left];
        shaderJob.shader.data.inputRight.value = [right];

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
