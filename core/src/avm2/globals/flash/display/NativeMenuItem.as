package flash.display {
    import flash.events.EventDispatcher;
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    // According to the documentation, it should be [API("661")]
    // but airglobal.swc disagrees with that:
    [API("667")]
    public class NativeMenuItem extends EventDispatcher {
        public var enabled: Boolean = false;
        public var checked: Boolean = false;
        public var data: Object;
        public var isSeparator: Boolean;
        public var keyEquivalent: String = "k";
        public var keyEquivalentModifiers: Array = [];
        public var label: String;
        public var mnemonicIndex: int = 0;
        public var name: String = "";
        public var submenu: NativeMenu = new NativeMenu();

        public function NativeMenuItem(label:String = "", isSeparator:Boolean = false)
        {
            stub_constructor("flash.display.NativeMenuItem");
            this.label = label;
            this.isSeparator = isSeparator;
        }

        public function get menu():NativeMenu
        {
            stub_getter("flash.display.NativeMenuItem", "menu");
            return new NativeMenu();
        }
    }
}
