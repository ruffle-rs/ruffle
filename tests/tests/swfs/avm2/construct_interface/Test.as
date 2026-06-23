package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            try {
                var a:ITest = new ITest();
                trace("Construction should not succeed!");
            }
            catch(e:VerifyError) {
                trace("Caught error");
                trace(e);
                trace(e.errorID);
            }
        }
    }
}
