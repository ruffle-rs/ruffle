package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "padding_bytes.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        var byteIndices = [
            // if
            0x38, 0x39, 0x3a, 0x3e,
            // else
            0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e,
            // endif
            0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e,
            // select
            0x66, 0x6a, 0x6e,
            // mov
            0x76
        ];

        for each (var index in byteIndices) {
            var bytes = new ShaderBytes();

            bytes.position = index;
            bytes.writeByte(1);
            testShader(bytes, index);
        }
    }

    private function testShader(bytes:*, i:Number) {
        var shader;
        try {
            shader = new Shader(bytes);
        } catch (e) {
            trace(i + ": " + e.getStackTrace());
            return;
        }

        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(shader, input);

        try {
            shaderJob.start(true);
            trace(i + ": " + input.getPixel32(0, 0).toString(16));
        } catch (e) {
            trace(i + ": Error while starting: " + e.getStackTrace());
        }
    }
}
}
