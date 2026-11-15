package flash.display {
    import flash.events.EventDispatcher;

    // According to the documentation, it should be [API("661")]
    // but airglobal.swc disagrees with that:
    [API("667")]
    public class NativeMenu extends EventDispatcher {
        // The array of NativeMenuItem objects in this menu.
        private var _items:Array;

        // The parent menu.
        private var _parent:NativeMenu;

        public function NativeMenu() {}

        // All methods are gated to AIR

        [API("668")]
        public function get items():Array {
            if (this._items == null) {
                this._items = [];
            }

            return this._items.AS3::concat();
        }
        [API("668")]
        public function set items(newItems:Array):void {
            this.removeAllItems();
            for (var i:int = 0; i < newItems.length; i ++) {
                this.addItem(newItems[i]);
            }
        }

        // Adds a menu item at the bottom of the menu.
        [API("668")]
        public function addItem(item:NativeMenuItem):NativeMenuItem {
            return this.addItemAt(item, this.numItems);
        }

        // Inserts a menu item at the specified position.
        [API("668")]
        public function addItemAt(item:NativeMenuItem, index:int):NativeMenuItem {
            if (this._items == null) {
                this._items = [];
            }

            if (index < 0 || index > this.numItems) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            if (item == null) {
                // For some reason this is an `ArgumentError` here instead of a `TypeError`
                throw new ArgumentError("Error #2007: Parameter item must be non-null.", 2007);
            }

            this._items.splice(index, 0, item);
            return item;
        }

        // Adds a submenu to the menu by inserting a new menu item.
        [API("668")]
        public function addSubmenu(submenu:NativeMenu, label:String):NativeMenuItem {
            return this.addSubmenuAt(submenu, this.numItems, label);
        }

        // Adds a submenu to the menu by inserting a new menu item at the specified position.
        [API("668")]
        public function addSubmenuAt(submenu:NativeMenu, index:int, label:String):NativeMenuItem {
            var item:NativeMenuItem = new NativeMenuItem(label);
            item.submenu = submenu;
            return this.addItemAt(item, index);
        }

        // Creates a copy of the menu and all items.
        [API("668")]
        public function clone():NativeMenu {
            var copy:NativeMenu = new NativeMenu();
            if (this._items != null) {
                copy._items = this._items.AS3::concat();
            }
            return copy;
        }

        // Reports whether this menu contains the specified menu item.
        [API("668")]
        public function containsItem(item:NativeMenuItem):Boolean {
            return this.getItemIndex(item) >= 0;
        }

        // Pops up this menu at the specified location.
        [API("668")]
        public function display(stage:Stage, stageX:Number, stageY:Number):void {
        }

        // Gets the menu item at the specified index.
        [API("668")]
        public function getItemAt(index:int):NativeMenuItem {
            if (index < 0 || index >= this.numItems) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            return this._items[index];
        }

        // Gets the menu item with the specified name.
        [API("668")]
        public function getItemByName(name:String):NativeMenuItem {
            if (this._items != null) {
                for (var i:int = 0; i < this._items.length; i++) {
                    if (this._items[i].name == name) {
                        return this._items[i];
                    }
                }
            }
            return null;
        }

        // Gets the position of the specified item.
        [API("668")]
        public function getItemIndex(item:NativeMenuItem):int {
            if (this._items == null) {
                return -1;
            }
            return this._items.indexOf(item);
        }

        // Removes all items from the menu.
        [API("668")]
        public function removeAllItems():void {
            this._items = [];
        }

        // Removes the specified menu item.
        [API("668")]
        public function removeItem(item:NativeMenuItem):NativeMenuItem {
            var idx:int = this.getItemIndex(item);
            if (idx < 0) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }
            return this.removeItemAt(idx);
        }

        // Removes and returns the menu item at the specified index.
        [API("668")]
        public function removeItemAt(index:int):NativeMenuItem {
            if (index < 0 || index >= this.numItems) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            return this._items.splice(index, 1)[0];
        }

        // Moves a menu item to the specified position.
        [API("668")]
        public function setItemIndex(item:NativeMenuItem, index:int):void {
            var currentIdx:int = this.getItemIndex(item);
            if (currentIdx < 0) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }
            if (index < 0 || index >= this.numItems) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            this._items.splice(currentIdx, 1);
            this._items.splice(index, 0, item);
        }

        [API("668")]
        public function get isSupported():Boolean {
            return false;
        }

        [API("668")]
        public function get numItems():int {
            if (this._items == null) {
                this._items = [];
            }

            return this._items.length;
        }

        [API("668")]
        public function get parent():NativeMenu {
            return this._parent;
        }
    }
}
