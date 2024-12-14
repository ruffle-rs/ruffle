package flash.ui {
    public final class ContextMenuClipboardItems {
        private var _clear:Boolean;
        private var _copy:Boolean;
        private var _cut:Boolean;
        private var _paste:Boolean;
        private var _selectAll:Boolean;
        
        public function get clear():Boolean {
            return this._clear;
        }
        
        public function set clear(value:Boolean):void {
            this._clear = value;
        }
        
        public function get copy():Boolean {
            return this._copy;
        }
        
        public function set copy(value:Boolean):void {
            this._copy = value;
        }
        
        public function get cut():Boolean {
            return this._cut;
        }
        
        public function set cut(value:Boolean):void {
            this._cut = value;
        }
        
        public function get paste():Boolean {
            return this._paste;
        }
        
        public function set paste(value:Boolean):void {
            this._paste = value;
        }
        
        public function get selectAll():Boolean {
            return this._selectAll;
        }
        
        public function set selectAll(value:Boolean):void {
            this._selectAll = value;
        }
        
        public function clone():ContextMenuClipboardItems {
            var items:ContextMenuClipboardItems = new ContextMenuClipboardItems();
            items.clear = this.clear;
            items.copy = this.copy;
            items.cut = this.cut;
            items.paste = this.paste;
            items.selectAll = this.selectAll;
            return items;
        }
    }
}
