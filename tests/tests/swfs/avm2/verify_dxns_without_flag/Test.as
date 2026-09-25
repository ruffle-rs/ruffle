// NOTE: this test requires manually patching the compiled SWF to clear the
// SET_DXNS method flag on `dxnsMethod`, since no compiler will normally emit
// a `dxns`/`dxnslate` opcode without also setting that flag.

package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            try {
                dxnsMethod();
                trace("Call should not succeed!");
            }
            catch (e:VerifyError) {
                trace("Caught error");
                trace(e);
                trace(e.errorID);
            }
        }

        private function dxnsMethod():void {
            default xml namespace = "test";
        }
    }
}
