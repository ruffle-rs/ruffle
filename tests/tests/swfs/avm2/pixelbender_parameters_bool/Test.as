package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "bools.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        testParameter(0, function(data) {
            data.pBool.value = [0];
        });
        testParameter(0, function(data) {
            data.pBool.value = [1];
        });
        testParameter(0, function(data) {
            data.pBool.value = [-1];
        });
        testParameter(0, function(data) {
            data.pBool.value = [0.1];
        });
        testParameter(0, function(data) {
            data.pBool.value = [0.5];
        });
        testParameter(0, function(data) {
            data.pBool.value = [0.9];
        });
        testParameter(0, function(data) {
            data.pBool.value = [1.0];
        });
        testParameter(0, function(data) {
            data.pBool.value = [1.1];
        });
        testParameter(0, function(data) {
            data.pBool.value = [2];
        });
        testParameter(0, function(data) {
            data.pBool.value = [3];
        });
        testParameter(0, function(data) {
            data.pBool.value = [-0.1];
        });
        testParameter(0, function(data) {
            data.pBool.value = [-0.5];
        });
        testParameter(0, function(data) {
            data.pBool.value = [-0.9];
        });
        testParameter(0, function(data) {
            data.pBool.value = [-1.0];
        });
        testParameter(0, function(data) {
            data.pBool.value = [-1.1];
        });
        testParameter(0, function(data) {
            data.pBool.value = [-2];
        });
        testParameter(0, function(data) {
            data.pBool.value = [-3];
        });

        testParameter(1, function(data) {
            data.pBool2.value = [];
        });
        testParameter(1, function(data) {
            data.pBool2.value = [0];
        });
        testParameter(1, function(data) {
            data.pBool2.value = [1];
        });
        testParameter(1, function(data) {
            data.pBool2.value = [0, 1];
        });
        testParameter(1, function(data) {
            data.pBool2.value = [1, 0];
        });
        testParameter(1, function(data) {
            data.pBool2.value = [1, 1];
        });
        testParameter(1, function(data) {
            data.pBool2.value = [0, 0];
        });
    }

    private function testParameter(selector:Number, setter:Function) {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);
        trace("Default params:");
        printParams(shaderJob.shader.data);

        setter(shaderJob.shader.data);
        shaderJob.shader.data.selector.value = [selector];

        trace("After setting params:");
        printParams(shaderJob.shader.data);

        try {
            shaderJob.start(true);
            trace("Result: " + input.getPixel32(0, 0).toString(16));
        } catch (e) {
            trace("Error while starting: " + e);
        }
        trace("=================");
    }

    private function printParams(data) {
        trace("  selector=" + data.selector.value);
        trace("  pBool=" + data.pBool.value);
        trace("  pBool2=" + data.pBool2.value);
    }
}
}
