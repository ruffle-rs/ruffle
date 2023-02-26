package flash.display {

	import flash.ui.ContextMenu;

	[Ruffle(NativeInstanceInit)]
	public class InteractiveObject extends DisplayObject {
		public function InteractiveObject() {
			throw new Error("You cannot directly construct InteractiveObject.")
		}

		public native function get mouseEnabled():Boolean;
		public native function set mouseEnabled(value:Boolean):void;

		public native function get doubleClickEnabled():Boolean;
		public native function set doubleClickEnabled(value:Boolean):void;

		public native function get contextMenu():ContextMenu;
		public native function set contextMenu(cm:ContextMenu):void;

		public native function get tabEnabled():Boolean;
		public native function set tabEnabled(value:Boolean):void;

		public native function get tabIndex():int;
		public native function set tabIndex(index:int):void;

		public native function get focusRect():Object;
		public native function set focusRect(value:Object):void;
	}
}
