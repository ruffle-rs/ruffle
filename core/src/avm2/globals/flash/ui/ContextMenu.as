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

        public var customItems: Array;

        public native function hideBuiltInItems(): void;

        public static function get isSupported() : Boolean
        {
            // TODO: return true when implementation actually affects the context menu
            return false;
        }
    }
}
