package flash.ui
{
    import flash.display.NativeMenu;
    import __ruffle__.stub_getter;

    public final class ContextMenu extends NativeMenu
    {
        public function ContextMenu()
        {
            super();
            this.customItems = new Array();
        }

        private var _customItems:Array;
        private var _clipboardMenu:Boolean;
        
        public function get customItems():Array {
            return this._customItems;
        }
        
        public function set customItems(value:Array):void {
            this._customItems = value;
        }

        public native function hideBuiltInItems(): void;

        private var _builtInItems: ContextMenuBuiltInItems = new ContextMenuBuiltInItems();
        private var _clipboardItems: ContextMenuClipboardItems = new ContextMenuClipboardItems();

        public function get builtInItems(): ContextMenuBuiltInItems {
            return this._builtInItems;
        }

        public function set builtInItems(value:ContextMenuBuiltInItems):void {
            this._builtInItems = value;
        }

        public function get clipboardItems(): ContextMenuClipboardItems {
            return this._clipboardItems;
        }

        public function set clipboardItems(value:ContextMenuClipboardItems):void {
            this._clipboardItems = value;
        }

        public function get clipboardMenu():Boolean {
            return _clipboardMenu;
        }

        public function set clipboardMenu(value:Boolean):void {
            _clipboardMenu = value;
        }

        public static function get isSupported():Boolean {
            // TODO: return true when implementation actually affects the context menu
            stub_getter("flash.ui.ContextMenu", "isSupported");
            return false;
        }
    }
}
