package {
    import flash.display.Sprite;
    import flash.display.Shader;
    import flash.display.ShaderJob;
    import flash.events.Event;
    import flash.events.ShaderEvent;
    import flash.utils.ByteArray;

    /*
     * `ShaderJob.start()` must dispatch an `Event.COMPLETE` event once the
     * shader has finished running, regardless of `waitForCompletion`.
     * ActionScript that kicks off a job and reads the result from the
     * `COMPLETE` handler would otherwise never see the event, leaving its
     * target empty.
     *
     * Nothing is traced outside of the event handler, so the output of this
     * test does not depend on whether the job happens to run synchronously.
     */
    public class Test extends Sprite {
        // The contents of `shader.pbj`, which was assembled from
        // `shader.pbasm` (see `pixelbender_ceil` for the assembly syntax).
        // It is inlined here because the test is recompiled from ActionScript,
        // and `[Embed]` is not supported by the recompiler.
        private static const SHADER_BYTES_HEX:String =
            "a501000000a406006365696c5f69a10101010008666c6f6174496e70757400a1020400000f64737400320200"
            + "103ecccccd32020020428000001b010040010000001d000080010000001d000040010040001d000020010040"
            + "000500008002008000010000800200c0000500004002008000010000400200c0000300002002008000010000"
            + "200200c000320000103f800000";

        public function Test() {
            testJob(true);
            testJob(false);
        }

        private function testJob(waitForCompletion:Boolean):void {
            var bytecode:ByteArray = new ByteArray();
            for (var i:int = 0; i < SHADER_BYTES_HEX.length; i += 2) {
                bytecode.writeByte(parseInt(SHADER_BYTES_HEX.substr(i, 2), 16));
            }

            var target:ByteArray = new ByteArray();
            var job:ShaderJob = new ShaderJob(new Shader(bytecode), target, 1, 1);
            job.addEventListener(Event.COMPLETE, function (event:Event):void {
                var shaderEvent:ShaderEvent = event as ShaderEvent;
                trace("waitForCompletion=" + waitForCompletion + ", type=" + event.type
                    + ", isShaderEvent=" + (shaderEvent != null));
                trace("  byteArray is target: " + (shaderEvent != null && shaderEvent.byteArray === target));
                trace("  bitmapData: " + (shaderEvent == null ? null : shaderEvent.bitmapData)
                    + ", vector: " + (shaderEvent == null ? null : shaderEvent.vector));
                trace("  target: " + toHex(target));
            });
            job.start(waitForCompletion);
        }

        private function toHex(bytes:ByteArray):String {
            var result:String = "";
            for (var i:int = 0; i < bytes.length; i++) {
                result += ("0" + bytes[i].toString(16)).substr(-2);
            }
            return result;
        }
    }
}
