package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "multiple_out.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;

    public function Test() {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(new Shader(new ShaderBytes()), input);

        shaderJob.start(true);
        trace(input.getPixel32(0, 0).toString(16));
    }
}
}
