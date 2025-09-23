package flash.events {
    import flash.filesystem.File;

    [API("661")]
    public class InvokeEvent extends Event {
        public static const INVOKE:String = "invoke";

        private var _arguments:Array;
        private var _reason:String;
        private var _currentDirectory:File;

        public function InvokeEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            dir:File = null,
            argv:Array = null,
            reason:String = "standard"
        ) {
            super(type, bubbles, cancelable);
            this._currentDirectory = dir;
            this._arguments = argv;
            this._reason = reason;
        }

        override public function clone():Event {
            return new InvokeEvent(this.type, this.bubbles, this.cancelable, this._currentDirectory, this._arguments, this._reason);
        }

        public function get arguments():Array {
            return this._arguments;
        }

        [API("664")]
        public function get reason():String {
            return this._reason;
        }

        public function get currentDirectory():File {
            return this._currentDirectory;
        }
    }
}
