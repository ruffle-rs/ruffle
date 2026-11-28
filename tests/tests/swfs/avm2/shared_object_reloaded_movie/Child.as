package {
    import flash.display.Sprite;
    import flash.net.SharedObject;

    public class Child extends Sprite {
        public function getLocal(name:String, localPath:String):SharedObject {
            return SharedObject.getLocal(name, localPath);
        }
    }
}
