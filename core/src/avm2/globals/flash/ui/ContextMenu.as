package flash.ui
{
    import flash.display.NativeMenu;
    import __ruffle__.stub_setter;

    public final class ContextMenu extends NativeMenu
    {
        public function ContextMenu()
        {
            super();
            this.customItems = new Array();
        }

        public var customItems: Array;

        public native function hideBuiltInItems(): void;

        private var _builtInItems: ContextMenuBuiltInItems = new ContextMenuBuiltInItems();

        public function get builtInItems(): ContextMenuBuiltInItems {
            return this._builtInItems;
        }

        public function set builtInItems(value:ContextMenuBuiltInItems):void {
            this._builtInItems = value;
            stub_setter("flash.ui.ContextMenu", "builtInItems");
        }

        public static function get isSupported() : Boolean
        {
            // TODO: return true when implementation actually affects the context menu
            return false;
        }
    }
}
