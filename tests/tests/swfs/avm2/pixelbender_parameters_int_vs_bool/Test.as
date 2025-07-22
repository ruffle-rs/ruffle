package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "params.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        testValue(0.0);
        testValue(0.1);
        testValue(0.5);
        testValue(0.9);
        testValue(1.0);
        testValue(1.1);
        testValue(1.5);
        testValue(1.9);
        testValue(2.0);
        testValue(2.5);
        testValue(-0.1);
        testValue(-0.5);
        testValue(-0.9);
        testValue(-1.0);
        testValue(-1.1);
        testValue(-1.5);
        testValue(-2.0);
        testValue(-2.5);
    }

    private function testValue(value:Number) {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);

        shaderJob.shader.data.boolInput.value = [value];
        shaderJob.shader.data.intInput.value = [value];

        try {
            shaderJob.start(true);
            trace("Value: " + value);
            trace("Result: " + input.getPixel32(0, 0).toString(16));
        } catch (e) {
            trace("Error while starting: " + e);
        }
        trace("=================");
    }
}
}
