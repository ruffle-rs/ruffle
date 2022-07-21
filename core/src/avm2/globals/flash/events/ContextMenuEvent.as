package flash.events {
	import flash.display.InteractiveObject;
	public class ContextMenuEvent extends Event {
		public static const MENU_ITEM_SELECT:String = "menuItemSelect";
		public static const MENU_SELECT:String = "menuSelect";
       
		private var _mouseTarget:InteractiveObject;
		private var _contextMenuOwner:InteractiveObject;
		private var _isMouseTargetInaccessible:Boolean;

		public function ContextMenuEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, mouseTarget:InteractiveObject = null, contextMenuOwner:InteractiveObject = null) {
			super(type,bubbles,cancelable);
			this._mouseTarget = mouseTarget;
			this._contextMenuOwner = contextMenuOwner;
		}

		override public function clone() : Event {
			return new ContextMenuEvent(this.type, this.bubbles, this.cancelable, this._mouseTarget, this._contextMenuOwner);
		}

		override public function toString() : String {
 			return this.formatToString("ContextMenuEvent","type","bubbles","cancelable","eventPhase","mouseTarget","contextMenuOwner");
		}

		public function get mouseTarget() : InteractiveObject {
			return this._mouseTarget;
		}

		public function set mouseTarget(value:InteractiveObject) : void {
			this._mouseTarget = value;
		}

		public function get contextMenuOwner() : InteractiveObject {
			return this._contextMenuOwner;
		}

		public function set contextMenuOwner(value:InteractiveObject) : void {
			this._contextMenuOwner = value;
		}

		public function get isMouseTargetInaccessible() : Boolean {
			return this._isMouseTargetInaccessible;
		}

		public function set isMouseTargetInaccessible(value:Boolean) : void {
			this._isMouseTargetInaccessible = value;
		}
	}
}
