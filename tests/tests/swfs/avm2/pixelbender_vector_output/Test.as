package {
    import flash.display.Shader;
    import flash.display.ShaderJob;
    import flash.display.MovieClip;

    public class Test extends MovieClip {

        [Embed(source = "vector_test_4ch.pbj", mimeType="application/octet-stream")]
        public static var SHADER_4CH:Class;

        [Embed(source = "vector_test_3ch.pbj", mimeType="application/octet-stream")]
        public static var SHADER_3CH:Class;

        public function Test() {
            test4Channel();
            test3Channel();
        }

        private function test4Channel():void {
            trace("=== 4-channel (pixel4) test ===");
            var shader:Shader = new Shader(new SHADER_4CH());

            shader.data["param1"].value = [5.0];
            shader.data["param2"].value = [3.0];

            var result:Vector.<Number> = new Vector.<Number>();
            var job:ShaderJob = new ShaderJob(shader, result, 1, 1);
            job.start(true);

            trace("Result length: " + result.length);
            for (var i:int = 0; i < result.length; i++) {
                trace("result[" + i + "] = " + result[i]);
            }
        }

        private function test3Channel():void {
            trace("=== 3-channel (pixel3) test ===");
            var shader:Shader = new Shader(new SHADER_3CH());

            shader.data["param1"].value = [5.0];
            shader.data["param2"].value = [3.0];

            var result:Vector.<Number> = new Vector.<Number>();
            var job:ShaderJob = new ShaderJob(shader, result, 1, 1);
            job.start(true);

            trace("Result length: " + result.length);
            for (var i:int = 0; i < result.length; i++) {
                trace("result[" + i + "] = " + result[i]);
            }
        }
    }
}
