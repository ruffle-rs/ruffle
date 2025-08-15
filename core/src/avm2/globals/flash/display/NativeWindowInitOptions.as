package flash.display {
    [API("661")]
    public class NativeWindowInitOptions {
        public var maximizable:Boolean;
        public var minimizable:Boolean;
        [API("671")]
        public var owner:NativeWindow;
        [API("675")]
        public var renderMode:String;
        public var resizable:Boolean;
        public var systemChrome:String;
        public var transparent:Boolean;
        public var type:String;

        public function NativeWindowInitOptions() {
            systemChrome = NativeWindowSystemChrome.STANDARD;
            type = NativeWindowType.NORMAL;
            transparent = false;
            owner = null;
            resizable = true;
            maximizable = true;
            minimizable = true;
        }
    }
}
