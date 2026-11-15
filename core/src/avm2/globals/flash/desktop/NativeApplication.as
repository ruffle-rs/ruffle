package flash.desktop {
    import flash.display.NativeWindow;
    import flash.events.InvokeEvent;
    import flash.events.Event;
    import flash.events.TimerEvent;
    import flash.events.EventDispatcher;
    import flash.system.Security;
    import flash.utils.Timer;
    import flash.utils.setTimeout;

    [API("661")]
    public final class NativeApplication extends EventDispatcher {
        private static var _instance:NativeApplication;

        private var _openedWindows:Array = [];
        private var _idleThreshold:int = 300;
        private var _autoExit:Boolean = false;
        private var _systemIdleMode:String = "normal";
        private var _startAtLogin:Boolean = false;
        private var _executeInBackground:Boolean = false;

        public function NativeApplication() {
            super();
            setTimeout(function():void {
                dispatchEvent(new InvokeEvent(InvokeEvent.INVOKE, false, false, null, []));
            }, 500);
        }

        public static function get nativeApplication():NativeApplication {
            if (!_instance) {
                _instance = new NativeApplication();
            }
            return _instance;
        }

        public static function get supportsMenu():Boolean {
            return false;
        }

        public static function get supportsDockIcon():Boolean {
            return false;
        }

        public static function get supportsSystemTrayIcon():Boolean {
            return false;
        }

        [API("668")]
        public static function get supportsDefaultApplication():Boolean {
            return false;
        }

        [API("668")]
        public static function get supportsStartAtLogin():Boolean {
            return false;
        }

        public function exit(exitCode:int = 0):void {
        }

        public function get runtimeVersion():String {
            return "5.0.0";
        }

        public function get runtimePatchLevel():uint {
            return 0;
        }

        public function get applicationID():String {
            return "";
        }

        public function get publisherID():String {
            return "";
        }

        public function get applicationDescriptor():XML {
            // Return some generic info in the descriptor so that SWFs that read
            // from it don't error out
            var genericDescriptor:XML = <application xmlns="http://adobe.com"><id>com.example.Game</id><versionNumber>1.0.0</versionNumber></application>;
            return genericDescriptor;
        }

        public function get autoExit():Boolean {
            return this._autoExit;
        }

        public function set autoExit(param1:Boolean):void {
            this._autoExit = param1;
        }

        public function get icon():InteractiveIcon {
            return null;
        }

        [API("668")]
        public function get systemIdleMode():String {
            return this._systemIdleMode;
        }

        [API("668")]
        public function set systemIdleMode(param1:String):void {
            this._systemIdleMode = param1;
        }

        public function get startAtLogin():Boolean {
            return this._startAtLogin;
        }

        public function set startAtLogin(param1:Boolean):void {
            this._startAtLogin = param1;
        }

        public function activate(window:NativeWindow = null):void {
        }

        public function get activeWindow():NativeWindow {
            return _openedWindows[0];
        }

        public function get openedWindows():Array {
            return _openedWindows;
        }

        public function get timeSinceLastUserInput():int {
            return 100;
        }

        public function get idleThreshold():int {
            return this._idleThreshold;
        }

        public function set idleThreshold(value:int):void {
            this._idleThreshold = value;
        }

        public function copy():Boolean {
            return false;
        }

        public function cut():Boolean {
            return false;
        }

        public function paste():Boolean {
            return false;
        }

        public function clear():Boolean {
            return false;
        }

        public function selectAll():Boolean {
            return false;
        }

        public function getDefaultApplication(extension:String):String {
            return null;
        }

        public function isSetAsDefaultApplication(extension:String):Boolean {
            return true;
        }

        public function setAsDefaultApplication(extension:String):void {
        }

        public function removeAsDefaultApplication(extension:String):void {
        }

        [API("681")]
        public function get executeInBackground():Boolean {
            return this._executeInBackground;
        }

        [API("681")]
        public function set executeInBackground(param1:Boolean):void {
            this._executeInBackground = param1;
        }

        [API("681")]
        public function get isCompiledAOT():Boolean {
            return false;
        }

        [API("681")]
        public function get isActive():Boolean {
            return true;
        }
    }
}
