package flash.ui
{
    import flash.display.NativeMenu;

    public final class ContextMenu extends NativeMenu
    {
        public function ContextMenu()
        {
            super();
            this.customItems = new Array();
        }

        private var _customItems:Array;
        
        public function get customItems():Array {
            return this._customItems;
        }
        
        public function set customItems(value:Array):void {
            this._customItems = value;
        }

        public native function hideBuiltInItems(): void;

        private var _builtInItems: ContextMenuBuiltInItems = new ContextMenuBuiltInItems();

        public function get builtInItems(): ContextMenuBuiltInItems {
            return this._builtInItems;
        }

        public function set builtInItems(value:ContextMenuBuiltInItems):void {
            this._builtInItems = value;
        }

        public static function get isSupported() : Boolean
        {
            // TODO: return true when implementation actually affects the context menu
            return false;
        }
    }
}
