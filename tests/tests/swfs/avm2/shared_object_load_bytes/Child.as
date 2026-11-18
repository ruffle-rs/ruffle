package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.SharedObject;
    import flash.utils.ByteArray;

    public class Child extends Sprite {
        public function getLocal(name:String):SharedObject {
            return SharedObject.getLocal(name);
        }

        public function loadBytes(bytes:ByteArray, callback:Function):void {
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                callback(loader.content);
            });
            loader.loadBytes(bytes);
        }
    }
}
