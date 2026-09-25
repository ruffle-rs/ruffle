// NOTE: FFDEC, asc.jar, and Flash's compiler all refuse to compile this
// Actionscript directly, so compiling this test requires manually editing a SWF

package {
    import flash.display.MovieClip;

    public class Testa extends MovieClip {
        public function Testa() {
            try {
                this.abc();
                trace("Call should not succeed!");
            }
            catch(e:VerifyError) {
                trace("Caught error");
                trace(e);
                trace(e.errorID);
            }
        }

        public function abc():void;
    }
}
