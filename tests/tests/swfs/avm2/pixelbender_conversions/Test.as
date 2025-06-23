package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "conversions.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        testConversion(0.0);
        testConversion(0.1);
        testConversion(0.5);
        testConversion(0.9);
        testConversion(1.0);
        testConversion(1.1);
        testConversion(1.5);
        testConversion(1.9);
        testConversion(2.0);
        testConversion(2.5);
        testConversion(-0.1);
        testConversion(-0.5);
        testConversion(-0.9);
        testConversion(-1.0);
        testConversion(-1.1);
        testConversion(-1.5);
        testConversion(-2.0);
        testConversion(-2.5);
    }

    private function testConversion(value:Number) {
        testConversionFromBool(value);
        testConversionFromInt(value);
        testConversionFromFloat(value);
    }

    private function testConversionFromBool(value:Number) {
        trace("bool->float " + value);
        testParameter(1, function(data) {
            data.boolInput.value = [value];
        });
    }

    private function testConversionFromFloat(value:Number) {
        trace("float->bool " + value);
        testParameter(2, function(data) {
            data.floatInput.value = [value];
        });
        trace("float->int " + value);
        testParameter(5, function(data) {
            data.floatInput.value = [value];
        });
    }

    private function testConversionFromInt(value:Number) {
        trace("int->bool " + value);
        testParameter(3, function(data) {
            data.intInput.value = [value];
        });
        trace("int->float " + value);
        testParameter(4, function(data) {
            data.intInput.value = [value];
        });
    }

    private function testParameter(selection:Number, setter:Function) {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);

        setter(shaderJob.shader.data);
        shaderJob.shader.data.selection.value = [selection];

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
