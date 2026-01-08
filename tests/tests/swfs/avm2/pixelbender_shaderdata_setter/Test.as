package {
    import flash.display.Sprite;
    import flash.display.Shader;
    import flash.display.ShaderJob;
    import flash.display.ShaderParameter;
    import flash.display.ShaderInput;
    import flash.display.BitmapData;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        [Embed(source="passthrough.pbj", mimeType="application/octet-stream")]
        private static var PassthroughShader:Class;

        public function Test() {
            trace("=== Testing shader parameter values actually received by shader ===");
            trace("");

            // Test 1: Normal array value via .value
            trace("=== Test 1: Normal - shader.data.amount.value = [0.4] ===");
            runShader(function(shader:Shader):void {
                shader.data.amount.value = [0.4];
            });

            // Test 2: Array set directly on shader.data
            trace("");
            trace("=== Test 2: Array on data - shader.data.amount = [0.75] ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = [0.75];
            });

            // Test 3: Primitive number set directly
            trace("");
            trace("=== Test 3: Primitive number - shader.data.amount = 0.299 ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = 0.299;
            });

            // Test 4: Integer
            trace("");
            trace("=== Test 4: Integer - shader.data.amount = 1 ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = 1;
            });

            // Test 5: Zero
            trace("");
            trace("=== Test 5: Zero - shader.data.amount = 0 ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = 0;
            });

            // Test 6: Negative number
            trace("");
            trace("=== Test 6: Negative - shader.data.amount = -0.5 ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = -0.5;
            });

            // Test 7: Boolean true
            trace("");
            trace("=== Test 7: Boolean true - shader.data.amount = true ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = true;
            });

            // Test 8: Boolean false
            trace("");
            trace("=== Test 8: Boolean false - shader.data.amount = false ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = false;
            });

            // Test 9: String that looks like number
            trace("");
            trace("=== Test 9: String '0.123' - shader.data.amount = '0.123' ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = "0.123";
            });

            // Test 10: String non-numeric
            trace("");
            trace("=== Test 10: String 'hello' - shader.data.amount = 'hello' ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = "hello";
            });

            // Test 11: null
            trace("");
            trace("=== Test 11: null - shader.data.amount = null ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = null;
            });

            // Test 12: undefined
            trace("");
            trace("=== Test 12: undefined - shader.data.amount = undefined ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = undefined;
            });

            // Test 13: NaN
            trace("");
            trace("=== Test 13: NaN - shader.data.amount = NaN ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = NaN;
            });

            // Test 14: Infinity
            trace("");
            trace("=== Test 14: Infinity - shader.data.amount = Infinity ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = Infinity;
            });

            // Test 15: Empty array
            trace("");
            trace("=== Test 15: Empty array - shader.data.amount = [] ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = [];
            });

            // Test 16: Object
            trace("");
            trace("=== Test 16: Object - shader.data.amount = {valueOf: 0.777} ===");
            runShader(function(shader:Shader):void {
                shader.data.amount = {valueOf: function():Number { return 0.777; }};
            });

            trace("");
            trace("=== Done ===");
        }

        private function runShader(setup:Function):void {
            var shader:Shader = new Shader(new PassthroughShader() as ByteArray);
            var output:BitmapData = new BitmapData(1, 1, false, 0x000000);

            // Apply the test setup (sets the parameter)
            setup(shader);

            // Check what's stored
            trace("Stored value: " + shader.data.amount);
            trace("Stored type: " + typeof shader.data.amount);

            // Run the shader
            try {
                var job:ShaderJob = new ShaderJob(shader, output, 1, 1);
                job.start(true);

                // Read the output pixel - shader outputs amount as RGB
                var pixel:uint = output.getPixel(0, 0);
                var r:uint = (pixel >> 16) & 0xFF;
                var g:uint = (pixel >> 8) & 0xFF;
                var b:uint = pixel & 0xFF;

                // Convert back to float (0-255 -> 0.0-1.0)
                var receivedValue:Number = r / 255.0;
                trace("Output pixel RGB: " + r + ", " + g + ", " + b);
                trace("Shader received (approx): " + receivedValue.toFixed(4));
            } catch (e:Error) {
                trace("ERROR: " + e);
            }
        }
    }
}
