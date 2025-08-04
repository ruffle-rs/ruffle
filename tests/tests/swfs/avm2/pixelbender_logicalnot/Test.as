package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "logicalnot.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        testValues(0);
        testValues(1);
        testValues(2);
        testValues(51);
        testValues(32000);
        testValues(32767);
        testValues(-1);
        testValues(-2);
        testValues(-3);
        testValues(-4);
        testValues(-5);
        testValues(-6);
        testValues(-9);
        testValues(-10);
        testValues(-20);
        testValues(-30);
        testValues(-40);
        testValues(-1000);
        testValues(-32000);
        testValues(-32768);
    }

    private function testValues(value:*) {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);
        shaderJob.shader.data.intInput.value = [value];
        shaderJob.start(true);
        trace(value + " -> " + input.getPixel32(0, 0).toString(16));
    }
}
}
