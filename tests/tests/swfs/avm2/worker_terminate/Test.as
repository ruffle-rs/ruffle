package {
    import flash.display.MovieClip;
    import flash.system.Worker;
    import flash.system.WorkerDomain;
    import flash.utils.ByteArray;

    public class Test extends MovieClip {
        public function Test() {
            trace("== primordial ==");
            var p:Worker = Worker.current;
            trace("isPrimordial = " + p.isPrimordial);
            trace("state = " + p.state);
            tryTerminate("primordial", p);
            trace("state after = " + p.state);

            trace("== createWorker(null) ==");
            try {
                WorkerDomain.current.createWorker(null);
                trace("no error");
            } catch (e:Error) {
                trace(e.getStackTrace());
            }

            trace("== created ==");
            var w:Worker = WorkerDomain.current.createWorker(new ByteArray());
            trace("isPrimordial = " + w.isPrimordial);
            trace("state = " + w.state);

            tryTerminate("first", w);
            trace("state = " + w.state);

            tryTerminate("second", w);
            trace("state = " + w.state);
        }

        private function tryTerminate(label:String, w:Worker):void {
            try {
                var r:Boolean = w.terminate();
                trace(label + " terminate returned " + r);
            } catch (e:Error) {
                trace(label + " terminate threw:");
                trace(e.getStackTrace());
            }
        }
    }
}
