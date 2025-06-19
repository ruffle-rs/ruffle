package {
import flash.display.*;
import flash.geom.*;

[SWF(width="100", height="50", backgroundColor="#000000")]
public class Test extends MovieClip {
    [Embed(source = "shader.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;
    [Embed(source = "shader2.pbj", mimeType="application/octet-stream")]
    public static var Shader2Bytes: Class;

    public function Test() {
        renderBitmap(0, 0, [0]);
        renderBitmap(1, 0, [10]);
        renderBitmap(2, 0, []);

        testValue([1, 2, 3]); // too few
        testValue([1, 2, 3, 4, 5, 6]); // too many
        testValue([]); // empty
        testValue(["test"]); // wrong type
        testValue([undefined]); // wrong type
        testValue([null]); // wrong type
        testValue([new Object()]); // wrong type
        testValue([true]); // wrong type
        testValue([false]); // wrong type
        testValue([[1]]); // wrong type
        testValue([1, 2, 3, 4, "test"]); // wrong type as an additional param

        testValueInt([]); // too few
        testValueInt([1, 2]); // too many
        testValueInt(["test"]); // wrong type
        testValueInt([undefined]); // wrong type
        testValueInt([null]); // wrong type
        testValueInt([new Object()]); // wrong type
        testValueInt([true]); // wrong type
        testValueInt([false]); // wrong type
        testValueInt([1.65]); // wrong type
        testValueInt([[1]]); // wrong type
        testValueInt([1, "test"]); // wrong type as an additional param
    }

    public function testValue(value:Array) {
        trace("Testing value '" + value + "' (length " + value.length + ")");

        var input = new BitmapData(100, 100);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);

        trace("Default value");
        printData(shaderJob);

        shaderJob.shader.data.bBox.value = value;
        shaderJob.shader.data.src.input = input;

        trace("After setting");
        printData(shaderJob);

        try {
            shaderJob.start(true);
        } catch (e) {
            trace("Error while starting: " + e);
        }

        trace("After started");
        printData(shaderJob);

        trace("=================");
    }

    public function printData(shaderJob:ShaderJob) {
        trace("  data: " + shaderJob.shader.data.bBox.value);
    }

    public function testValueInt(value:Array) {
        trace("Testing int value '" + value + "' (length " + value.length + ")");

        var input = new BitmapData(100, 100);
        var shaderJob = new ShaderJob(new Shader(new Shader2Bytes()), input);

        trace("Default value");
        printData2(shaderJob);

        shaderJob.shader.data.oImage.input = input;
        shaderJob.shader.data.gaussOrSinc.value = value;

        trace("After setting");
        printData2(shaderJob);

        try {
            shaderJob.start(true);
        } catch (e) {
            trace("Error while starting: " + e);
        }

        trace("After started");
        printData2(shaderJob);

        trace("=================");
    }

    public function printData2(shaderJob:ShaderJob) {
        trace("  data: " + shaderJob.shader.data.gaussOrSinc.value);
    }

    public function renderBitmap(x:Number, y:Number, size:Array) {
        var input = new BitmapData(20, 20, false, 0x222222);
        var i = 0;
        while (i < 20) {
            input.fillRect(new Rectangle(i, 0, 1, 20), 0xFFFFFF);
            i += 2;
        }

        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);
        shaderJob.shader.data.bBox.value = [20, 20, 0, 50];
        shaderJob.shader.data.exponent.value = [-7.2];
        shaderJob.shader.data.factor.value = [-6.4];
        shaderJob.shader.data.center.value = [-1.12, 0.5];
        shaderJob.shader.data.size.value = size;
        shaderJob.shader.data.smudge.value = [0.38];
        shaderJob.shader.data.src.input = input;
        shaderJob.start(true);

        var bitmap = new Bitmap(input);
        bitmap.x = x * (bitmap.width + 5);
        bitmap.y = y * (bitmap.height + 5);
        addChild(bitmap);
    }
}
}
