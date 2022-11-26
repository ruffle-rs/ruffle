// This is a stub - the actual class is defined in `displayobjectcontainer.rs`
package flash.display {
	public class DisplayObjectContainer extends InteractiveObject {
		public native function addChild(child:DisplayObject):DisplayObject;
		public native function addChildAt(child:DisplayObject, index:int):DisplayObject;
		public native function removeChild(child:DisplayObject, index:int):DisplayObject;
		public native function removeChildAt(index:int):DisplayObject;
		public native function setChildIndex(child:DisplayObject, index:int):void;
		public native function getChildAt(index:int):DisplayObject;
	}
}
