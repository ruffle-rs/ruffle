package flash.display {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;

    import flash.events.EventDispatcher;

    // According to the documentation, it should be [API("661")]
    // but airglobal.swc disagrees with that:
    [API("667")]
    public class NativeMenuItem extends EventDispatcher {
        [Ruffle(NativeAccessible)]
        private var _enabled:Boolean = false;

        private var _checked:Boolean = false;
        private var _data:Object;
        private var _isSeparator:Boolean;
        private var _keyEquivalent:String = "k";
        private var _keyEquivalentModifiers:Array = [];
        private var _label:String;
        private var _mnemonicIndex:int = 0;
        private var _name:String = "";
        private var _submenu:NativeMenu = new NativeMenu();

        public function NativeMenuItem(label:String = "", isSeparator:Boolean = false) {
            stub_constructor("flash.display.NativeMenuItem");
            this.label = label;
            this.isSeparator = isSeparator;
        }

        public function get enabled():Boolean {
            return this._enabled;
        }
        public function set enabled(value:Boolean):void {
            this._enabled = value;
        }

        // The rest of the properties are AIR-only

        [API("668")]
        public function get checked():Boolean {
            return this._checked;
        }
        [API("668")]
        public function set checked(value:Boolean):void {
            this._checked = value;
        }

        [API("668")]
        public function get data():Object {
            return this._data;
        }
        [API("668")]
        public function set data(value:Object):void {
            this._data = value;
        }

        [API("668")]
        public function get isSeparator():Boolean {
            return this._isSeparator;
        }
        [API("668")]
        public function set isSeparator(value:Boolean):void {
            this._isSeparator = value;
        }

        [API("668")]
        public function get keyEquivalent():String {
            return this._keyEquivalent;
        }
        [API("668")]
        public function set keyEquivalent(value:String):void {
            this._keyEquivalent = value;
        }

        [API("668")]
        public function get keyEquivalentModifiers():Array {
            return this._keyEquivalentModifiers;
        }
        [API("668")]
        public function set keyEquivalentModifiers(value:Array):void {
            this._keyEquivalentModifiers = value;
        }

        [API("668")]
        public function get label():String {
            return this._label;
        }
        [API("668")]
        public function set label(value:String):void {
            this._label = value;
        }

        [API("668")]
        public function get mnemonicIndex():int {
            return this._mnemonicIndex;
        }
        [API("668")]
        public function set mnemonicIndex(value:int):void {
            // TODO validation
            this._mnemonicIndex = value;
        }

        [API("668")]
        public function get name():String {
            return this._name;
        }
        [API("668")]
        public function set name(value:String):void {
            this._name = value;
        }

        [API("668")]
        public function get submenu():NativeMenu {
            return this._submenu;
        }
        [API("668")]
        public function set submenu(value:NativeMenu):void {
            this._submenu = value;
        }

        [API("668")]
        public function get menu():NativeMenu {
            stub_getter("flash.display.NativeMenuItem", "menu");
            return new NativeMenu();
        }
    }
}
