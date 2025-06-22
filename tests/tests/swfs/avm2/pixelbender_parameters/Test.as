package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "shader.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        testParameter(0, function(data, value) {
            trace("Default params:");
            printParams(data);
        });
        testParameter(1, function(data, value) {
            data.pBool.value = value;
        });
        testParameter(2, function(data, value) {
            data.pInt.value = value;
        });
        testParameter(3, function(data, value) {
            data.pFloat.value = value;
        });
        testParameter(4, function(data, value) {
            data.pPixel1.value = value;
        });
        testParameter(5, function(data, value) {
            data.pFloat2.value = value;
        });
        testParameter(6, function(data, value) {
            data.pFloat3.value = value;
        });
        testParameter(7, function(data, value) {
            data.pFloat4.value = value;
        });
        testParameter(8, function(data, value) {
            data.pBool2.value = value;
        });
        testParameter(9, function(data, value) {
            data.pBool3.value = value;
        });
        testParameter(10, function(data, value) {
            data.pBool4.value = value;
        });
        testParameter(11, function(data, value) {
            data.pInt2.value = value;
        });
        testParameter(12, function(data, value) {
            data.pInt3.value = value;
        });
        testParameter(13, function(data, value) {
            data.pInt4.value = value;
        });
        testParameter(14, function(data, value) {
            data.pPixel2.value = value;
        });
        testParameter(15, function(data, value) {
            data.pPixel3.value = value;
        });
        testParameter(16, function(data, value) {
            data.pPixel4.value = value;
        });
        testParameter(17, function(data, value) {
            data.pFloat2x2.value = value;
        });
        testParameter(18, function(data, value) {
            data.pFloat3x3.value = value;
        });
        testParameter(19, function(data, value) {
            data.pFloat4x4.value = value;
        });
    }

    private function testParameter(selector:Number, setter:Function) {
        testParameterValue(selector, setter, null);
        if (selector == 0) return;
        testParameterValue(selector, setter, undefined);
        testParameterValue(selector, setter, []);
        testParameterValue(selector, setter, [256]);
        testParameterValue(selector, setter, [256, 256]);
        testParameterValue(selector, setter, [256, 256, 256]);
        testParameterValue(selector, setter, [0, 0, 256]);
        testParameterValue(selector, setter, [0, 0, "test"]);
        testParameterValue(selector, setter, ["test"]);
        testParameterValue(selector, setter, [undefined]);
        testParameterValue(selector, setter, [null]);
        testParameterValue(selector, setter, [undefined, 256]);
        testParameterValue(selector, setter, [null, 256]);
        testParameterValue(selector, setter, [new Object()]);
        testParameterValue(selector, setter, [true]);
        testParameterValue(selector, setter, [false]);
        testParameterValue(selector, setter, [true, false]);
        testParameterValue(selector, setter, [true, false, true]);
        testParameterValue(selector, setter, [0.8]);
        testParameterValue(selector, setter, [0.2]);
        testParameterValue(selector, setter, [0.6]);
        testParameterValue(selector, setter, [1.1]);
        testParameterValue(selector, setter, [1.5]);
        testParameterValue(selector, setter, [0.8, 0.8]);
        testParameterValue(selector, setter, [0.8, 1, 1]);
        testParameterValue(selector, setter, [1, 0, 1]);
        testParameterValue(selector, setter, [[1]]);
    }

    private function testParameterValue(selector:Number, setter:Function, value:Array) {
        trace("Testing selector " + selector + ", value '" + value + "'");

        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);

        setter(shaderJob.shader.data, value);
        shaderJob.shader.data.selector.value = [selector];

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
        trace("  pInt=" + data.pInt.value);
        trace("  pFloat=" + data.pFloat.value);
        trace("  pPixel1=" + data.pPixel1.value);
        trace("  pFloat2=" + data.pFloat2.value);
        trace("  pFloat3=" + data.pFloat3.value);
        trace("  pFloat4=" + data.pFloat4.value);
        trace("  pBool2=" + data.pBool2.value);
        trace("  pBool3=" + data.pBool3.value);
        trace("  pBool4=" + data.pBool4.value);
        trace("  pInt2=" + data.pInt2.value);
        trace("  pInt3=" + data.pInt3.value);
        trace("  pInt4=" + data.pInt4.value);
        trace("  pPixel2=" + data.pPixel2.value);
        trace("  pPixel3=" + data.pPixel3.value);
        trace("  pPixel4=" + data.pPixel4.value);
        trace("  pFloat2x2=" + data.pFloat2x2.value);
        trace("  pFloat3x3=" + data.pFloat3x3.value);
        trace("  pFloat4x4=" + data.pFloat4x4.value);
    }
}
}
