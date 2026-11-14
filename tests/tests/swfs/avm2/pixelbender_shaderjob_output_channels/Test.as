// Runs a ShaderJob with `Vector.<Number>` and `ByteArray` targets for each
// shader output type (float, float2, float3, float4). Flash accepts every
// output type for non-BitmapData targets.
//
// Each shader divides its input image by `divisor`.
package {
    import flash.display.MovieClip;
    import flash.display.Shader;
    import flash.display.ShaderJob;
    import flash.utils.ByteArray;
    import flash.utils.Endian;

    public class Test extends MovieClip {
        [Embed(source = "opdiv_float.pbj", mimeType = "application/octet-stream")]
        private static const DivFloat:Class;

        [Embed(source = "opdiv_float2.pbj", mimeType = "application/octet-stream")]
        private static const DivFloat2:Class;

        [Embed(source = "opdiv_float3.pbj", mimeType = "application/octet-stream")]
        private static const DivFloat3:Class;

        [Embed(source = "opdiv_float4.pbj", mimeType = "application/octet-stream")]
        private static const DivFloat4:Class;

        // Divisible by every channel count, so each run covers whole pixels.
        private static const VALUE_COUNT:uint = 12;

        public function Test() {
            runDivision("float", new DivFloat(), 1);
            runDivision("float2", new DivFloat2(), 2);
            runDivision("float3", new DivFloat3(), 3);
            runDivision("float4", new DivFloat4(), 4);
        }

        private function runDivision(name:String, bytecode:ByteArray, channels:uint):void {
            var input:Vector.<Number> = new Vector.<Number>();
            for (var i:uint = 1; i <= VALUE_COUNT; i++) {
                input.push(i);
            }
            var width:uint = VALUE_COUNT / channels;

            var shader:Shader = new Shader(bytecode);
            shader.data.texture.input = input;
            shader.data.texture.width = width;
            shader.data.texture.height = 1;
            shader.data.divisor.value = [2];

            var vector:Vector.<Number> = new Vector.<Number>();
            try {
                new ShaderJob(shader, vector, width, 1).start(true);
                trace(name + " Vector: " + vector);
            } catch (e:Error) {
                trace(name + " Vector: " + e);
            }

            var bytes:ByteArray = new ByteArray();
            bytes.endian = Endian.LITTLE_ENDIAN;
            try {
                new ShaderJob(shader, bytes, width, 1).start(true);
                trace(name + " ByteArray: " + readFloats(bytes));
            } catch (e:Error) {
                trace(name + " ByteArray: " + e);
            }
        }

        private static function readFloats(bytes:ByteArray):String {
            var floats:Array = [];
            bytes.position = 0;
            while (bytes.bytesAvailable >= 4) {
                floats.push(bytes.readFloat());
            }
            return "length=" + bytes.length + " " + floats;
        }
    }
}
