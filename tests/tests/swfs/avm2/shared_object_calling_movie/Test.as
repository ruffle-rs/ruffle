package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.SharedObject;
    import flash.net.URLRequest;

    // SharedObject.getLocal called by bytecode of the root movie or of a
    // child movie, where the other movie is the caller, `this`, or the
    // dispatcher of an event. Each label names the movie whose bytecode
    // calls getLocal.
    public class Test extends Sprite {
        private var localPaths:Array = [null, "/"];
        private var objects:Array = [];
        private var labels:Array = [];
        private var child:Object;

        public function Test() {
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                child = loader.content;
                run();
            });
            loader.load(new URLRequest("sub/child.swf"));
        }

        private function getLocal(name:String, localPath:String):SharedObject {
            return SharedObject.getLocal(name, localPath);
        }

        private function run():void {
            trace("// Bytecode called by its own movie");
            check("root", getLocal);
            report(child.calledByChild, "child constructor, null");

            trace("// Bytecode called by the other movie");
            check("child method, called by root", child.getLocal);
            check("child closure, called by root", child.getLocalClosure);
            check("root method, called by child", function(name:String, localPath:String):SharedObject {
                return child.callGetLocal(getLocal, name, localPath);
            });
            var rootClosure:Function = function(name:String, localPath:String):SharedObject {
                return SharedObject.getLocal(name, localPath);
            };
            check("root closure, called by child", function(name:String, localPath:String):SharedObject {
                return child.callGetLocal(rootClosure, name, localPath);
            });

            trace("// Bytecode with the other movie's object as this");
            check("root closure, called with child as this", function(name:String, localPath:String):SharedObject {
                return rootClosure.call(child, name, localPath);
            });
            check("child closure, called with root as this", function(name:String, localPath:String):SharedObject {
                return child.getLocalClosure.call(this, name, localPath);
            });

            trace("// Root bytecode called back by the child");
            var calledBack:SharedObject;
            child.addEventListener("test", function(e:Event):void {
                calledBack = SharedObject.getLocal("so");
            });
            child.dispatch("test");
            report(calledBack, "root listener, dispatched by child, null");
        }

        private function check(label:String, getter:Function):void {
            for each (var localPath:String in localPaths) {
                report(getter("so", localPath), label + ", " + localPath);
            }
        }

        private function report(so:SharedObject, label:String):void {
            var index:int = objects.indexOf(so);
            if (index >= 0) {
                trace(label + ": same as " + labels[index]);
                return;
            }
            objects.push(so);
            labels.push(label);
            so.data.label = label;
            trace(label + ": new, " + so.flush());
        }
    }
}
