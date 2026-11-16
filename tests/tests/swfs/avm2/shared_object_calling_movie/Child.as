package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.SharedObject;

    public class Child extends Sprite {
        public var calledByChild:SharedObject = SharedObject.getLocal("so");

        public function getLocal(name:String, localPath:String):SharedObject {
            return SharedObject.getLocal(name, localPath);
        }

        public var getLocalClosure:Function = function(name:String, localPath:String):SharedObject {
            return SharedObject.getLocal(name, localPath);
        };

        public function callGetLocal(getter:Function, name:String, localPath:String):SharedObject {
            return getter(name, localPath);
        }

        public function dispatch(type:String):void {
            dispatchEvent(new Event(type));
        }
    }
}
