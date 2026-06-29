package {
    import flash.display.MovieClip;
    import flash.events.Event;
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
            p.addEventListener(Event.WORKER_STATE, function(e:Event):void {
                trace("primordial workerState event, state = " + p.state);
            });
            tryStart("primordial", p);
            trace("state after start = " + p.state);

            trace("== createWorker(null) ==");
            try {
                WorkerDomain.current.createWorker(null);
                trace("no error");
            } catch (e:Error) {
                trace(e.constructor);
                trace(e.getStackTrace());
            }

            trace("== terminate on New ==");
            var newWorker:Worker = WorkerDomain.current.createWorker(new ByteArray());
            listenForState("newWorker", newWorker);
            trace("isPrimordial = " + newWorker.isPrimordial);
            trace("state = " + newWorker.state);
            tryTerminate("first", newWorker);
            trace("state = " + newWorker.state);
            tryTerminate("second", newWorker);
            trace("state = " + newWorker.state);

            trace("== terminate on Terminated ==");
            var runWorker:Worker = WorkerDomain.current.createWorker(new ByteArray());
            listenForState("runWorker", runWorker, testStateEvents);
            tryStart("first", runWorker);
            tryStart("second", runWorker);
            trace("state after start = " + runWorker.state);
            tryTerminate("first", runWorker);
            trace("state = " + runWorker.state);
            tryTerminate("second", runWorker);
            trace("state = " + runWorker.state);
            tryStart("terminated", runWorker);
            trace("state = " + runWorker.state);
        }

        private function testStateEvents():void {
            trace("== workerState events ==");
            var worker:Worker = WorkerDomain.current.createWorker(new ByteArray());
            listenForState("worker", worker, function():void {
                if (worker.state == "running") {
                    tryTerminate("running", worker);
                }
            });
            tryStart("worker", worker);
            trace("state after start = " + worker.state);
        }

        private function listenForState(label:String, w:Worker, onEvent:Function = null):void {
            w.addEventListener(Event.WORKER_STATE, function(e:Event):void {
                trace(label + " workerState event: " + e + ", state = " + w.state);
                if (onEvent != null) {
                    onEvent();
                }
            });
        }

        private function tryStart(label:String, w:Worker):void {
            try {
                w.start();
                trace(label + " start returned");
            } catch (e:Error) {
                trace(label + " start threw:");
                trace(e.constructor);
                trace(e.getStackTrace());
            }
        }

        private function tryTerminate(label:String, w:Worker):void {
            try {
                var r:Boolean = w.terminate();
                trace(label + " terminate returned " + r);
            } catch (e:Error) {
                trace(label + " terminate threw:");
                trace(e.constructor);
                trace(e.getStackTrace());
            }
        }
    }
}
