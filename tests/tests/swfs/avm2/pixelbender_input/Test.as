package {
    import flash.display.Shader;
    import flash.display.ShaderJob;
    import flash.display.MovieClip;
    import flash.utils.ByteArray;
    import flash.utils.Endian;

    public class Test extends MovieClip {

        [Embed(source = "passthrough1.pbj", mimeType="application/octet-stream")]
        public static var SHADER1:Class;

        [Embed(source = "passthrough2.pbj", mimeType="application/octet-stream")]
        public static var SHADER2:Class;

        [Embed(source = "passthrough3.pbj", mimeType="application/octet-stream")]
        public static var SHADER3:Class;

        [Embed(source = "passthrough4.pbj", mimeType="application/octet-stream")]
        public static var SHADER4:Class;

        private const WIDTH:int = 2;
        private const HEIGHT:int = 2;

        public function Test() {
            // Test all channel counts with both input types
            testChannels(1, SHADER1);
            testChannels(2, SHADER2);
            testChannels(3, SHADER3);
            testChannels(4, SHADER4);
        }

        private function testChannels(channels:int, shaderClass:Class):void {
            trace("\n========== " + channels + " CHANNEL(S) ==========");

            var exactFloats:int = WIDTH * HEIGHT * channels;

            // Test with Vector.<Number>
            trace("\n--- Vector.<Number> ---");
            testVector(shaderClass, channels, exactFloats, "exact");
            testVector(shaderClass, channels, exactFloats + 1, "1 extra");
            testVector(shaderClass, channels, exactFloats + channels, channels + " extra (1 pixel)");
            testVector(shaderClass, channels, exactFloats - 1, "1 short");
            testVector(shaderClass, channels, exactFloats - channels, channels + " short (1 pixel)");

            // Test with ByteArray
            trace("\n--- ByteArray ---");
            testByteArray(shaderClass, channels, exactFloats, "exact");
            testByteArray(shaderClass, channels, exactFloats + 1, "1 extra");
            testByteArray(shaderClass, channels, exactFloats + channels, channels + " extra (1 pixel)");
            testByteArray(shaderClass, channels, exactFloats - 1, "1 short");
            testByteArray(shaderClass, channels, exactFloats - channels, channels + " short (1 pixel)");
        }

        private function createVector(numFloats:int):Vector.<Number> {
            var vec:Vector.<Number> = new Vector.<Number>();
            for (var i:int = 0; i < numFloats; i++) {
                vec.push((i + 1) * 0.1);
            }
            return vec;
        }

        private function createByteArray(numFloats:int):ByteArray {
            var ba:ByteArray = new ByteArray();
            ba.endian = Endian.LITTLE_ENDIAN;
            for (var i:int = 0; i < numFloats; i++) {
                ba.writeFloat((i + 1) * 0.1);
            }
            ba.position = 0;
            return ba;
        }

        private function testVector(shaderClass:Class, channels:int, numFloats:int, description:String):void {
            var exactFloats:int = WIDTH * HEIGHT * channels;
            trace("Vector " + description + " (" + numFloats + "/" + exactFloats + " floats)");

            var shader:Shader = new Shader(new shaderClass());
            var input:Vector.<Number> = createVector(numFloats);

            shader.data.src.input = input;
            shader.data.src.width = WIDTH;
            shader.data.src.height = HEIGHT;

            var result:Vector.<Number> = new Vector.<Number>();
            var job:ShaderJob = new ShaderJob(shader, result, WIDTH, HEIGHT);

            try {
                job.start(true);
                trace("  Success, output: " + result.length + " floats");
            } catch (e:Error) {
                trace("  Error: " + e.errorID);
            }
        }

        private function testByteArray(shaderClass:Class, channels:int, numFloats:int, description:String):void {
            var exactFloats:int = WIDTH * HEIGHT * channels;
            trace("ByteArray " + description + " (" + numFloats + "/" + exactFloats + " floats)");

            var shader:Shader = new Shader(new shaderClass());
            var input:ByteArray = createByteArray(numFloats);

            shader.data.src.input = input;
            shader.data.src.width = WIDTH;
            shader.data.src.height = HEIGHT;

            var result:Vector.<Number> = new Vector.<Number>();
            var job:ShaderJob = new ShaderJob(shader, result, WIDTH, HEIGHT);

            try {
                job.start(true);
                trace("  Success, output: " + result.length + " floats");
            } catch (e:Error) {
                trace("  Error: " + e.errorID);
            }
        }
    }
}
