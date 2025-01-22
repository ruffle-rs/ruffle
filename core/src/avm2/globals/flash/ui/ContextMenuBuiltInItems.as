package flash.ui {
    public final class ContextMenuBuiltInItems {
        [Ruffle(NativeAccessible)]
        private var _forwardAndBack:Boolean = true;

        [Ruffle(NativeAccessible)]
        private var _loop:Boolean = true;

        [Ruffle(NativeAccessible)]
        private var _play:Boolean = true;

        [Ruffle(NativeAccessible)]
        private var _print:Boolean = true;

        [Ruffle(NativeAccessible)]
        private var _quality:Boolean = true;

        [Ruffle(NativeAccessible)]
        private var _rewind:Boolean = true;

        [Ruffle(NativeAccessible)]
        private var _save:Boolean = true;

        [Ruffle(NativeAccessible)]
        private var _zoom:Boolean = true;
        
        public function get forwardAndBack():Boolean {
            return this._forwardAndBack;
        }
        
        public function set forwardAndBack(value:Boolean):void {
            this._forwardAndBack = value;
        }
        
        
        public function get loop():Boolean {
            return this._loop;
        }
        
        public function set loop(value:Boolean):void {
            this._loop = value;
        }
        
        
        public function get play():Boolean {
            return this._play;
        }
        
        public function set play(value:Boolean):void {
            this._play = value;
        }
        
        
        public function get print():Boolean {
            return this._print;
        }
        
        public function set print(value:Boolean):void {
            this._print = value;
        }
        
        
        public function get quality():Boolean {
            return this._quality;
        }
        
        public function set quality(value:Boolean):void {
            this._quality = value;
        }
        
        
        public function get rewind():Boolean {
            return this._rewind;
        }
        
        public function set rewind(value:Boolean):void {
            this._rewind = value;
        }
        
        
        public function get save():Boolean {
            return this._save;
        }
        
        public function set save(value:Boolean):void {
            this._save = value;
        }
        
        
        public function get zoom():Boolean {
            return this._zoom;
        }
        
        public function set zoom(value:Boolean):void {
            this._zoom = value;
        }

        public function clone():ContextMenuBuiltInItems {
            var items:ContextMenuBuiltInItems = new ContextMenuBuiltInItems();
            items.forwardAndBack = this.forwardAndBack;
            items.loop = this.loop;
            items.play = this.play;
            items.print = this.print;
            items.quality = this.quality;
            items.rewind = this.rewind;
            items.save = this.save;
            items.zoom = this.zoom;
            return items;
        }
    }
}
