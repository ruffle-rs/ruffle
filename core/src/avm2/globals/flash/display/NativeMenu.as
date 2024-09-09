package flash.display
{

    import flash.events.EventDispatcher;
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    
    // According to the documentation, it should be [API("661")]
    // but airglobal.swc disagrees with that:
    [API("667")]
    public class NativeMenu extends EventDispatcher
    {

        // Indicates whether any form of native menu is supported on the client system.
        private var _isSupported:Boolean;

        // The array of NativeMenuItem objects in this menu.
        public var items:Array;

        // The parent menu.
        private var _parent:NativeMenu;

        public function NativeMenu()
        {

        }

        // Adds a menu item at the bottom of the menu.
        public function addItem(item:NativeMenuItem):NativeMenuItem
        {
            stub_method("flash.display.NativeMenu", "addItem");
            this.items.push(item);
            return item;
        }

        // Inserts a menu item at the specified position.
        public function addItemAt(item:NativeMenuItem, index:int):NativeMenuItem
        {
            stub_method("flash.display.NativeMenu", "addItemAt");
            this.items[index] = item;
            return item;
        }

        // Adds a submenu to the menu by inserting a new menu item.
        public function addSubmenu(submenu:NativeMenu, label:String):NativeMenuItem
        {
            stub_method("flash.display.NativeMenu", "addSubmenu");
            return null;
        }

        // Adds a submenu to the menu by inserting a new menu item at the specified position.
        public function addSubmenuAt(submenu:NativeMenu, index:int, label:String):NativeMenuItem
        {
            stub_method("flash.display.NativeMenu", "addSubmenuAt");
            return null;
        }

        // Creates a copy of the menu and all items.
        public function clone():NativeMenu
        {
            stub_method("flash.display.NativeMenu", "clone");
            return null;
        }

        // Reports whether this menu contains the specified menu item.
        public function containsItem(item:NativeMenuItem):Boolean
        {
            stub_method("flash.display.NativeMenu", "containsItem");
            return false;
        }

        // Pops up this menu at the specified location.
        public function display(stage:Stage, stageX:Number, stageY:Number):void
        {
            stub_method("flash.display.NativeMenu", "display");
        }

        // Gets the menu item at the specified index.
        public function getItemAt(index:int):NativeMenuItem
        {
            stub_method("flash.display.NativeMenu", "getItemAt");
            return null;
        }

        // Gets the menu item with the specified name.
        public function getItemByName(name:String):NativeMenuItem
        {
            stub_method("flash.display.NativeMenu", "getItemByName");
            return null;
        }

        // Gets the position of the specified item.
        public function getItemIndex(item:NativeMenuItem):int
        {
            stub_method("flash.display.NativeMenu", "getItemIndex");
            return -1;
        }

        // Removes all items from the menu.
        public function removeAllItems():void
        {
            stub_method("flash.display.NativeMenu", "removeAllItems");
            this.items = [];
        }

        // Removes the specified menu item.
        public function removeItem(item:NativeMenuItem):NativeMenuItem
        {
            stub_method("flash.display.NativeMenu", "removeItem");
            return null;
        }

        // Removes and returns the menu item at the specified index.
        public function removeItemAt(index:int):NativeMenuItem
        {
            stub_method("flash.display.NativeMenu", "removeItemAt");
            return null;
        }

        // Moves a menu item to the specified position.
        public function setItemIndex(item:NativeMenuItem, index:int):void
        {
            stub_method("flash.display.NativeMenu", "setItemIndex");
        }

        // According to the documentation, it should be [API("668")]
        // but there is no version gate in airglobal.swc
        public function get isSupported():Boolean
        {
            return this._isSupported;
        }

        public function get numItems():int
        {
            return this.items.length;
        }

        public function get parent():NativeMenu
        {
            return this._parent;
        }

    }
}
