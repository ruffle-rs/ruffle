package flash.display {
	public class Loader extends DisplayObjectContainer {
		import flash.display.LoaderInfo;
		import flash.display.DisplayObject;
		import flash.errors.IllegalOperationError;
		import flash.system.LoaderContext;
		import flash.utils.ByteArray;
		import flash.net.URLRequest;
   		import __ruffle__.log_warn;

		private var _contentLoaderInfo: LoaderInfo;

		public function get contentLoaderInfo():LoaderInfo {
			return this._contentLoaderInfo;
		}

		private native function init();

		public function Loader() {
			this.init()
		}

		public function get content():DisplayObject {
			if (this.numChildren == 0) {
				return null;
			}
			return this.getChildAt(0)
		}

		public native function load(request: URLRequest, context: LoaderContext = null):void;

		public native function loadBytes(data: ByteArray, context: LoaderContext = null):void;
		
		public function unload():void {
			log_warn("flash.display.Loader.unload - not yet implemented");
		}

		override public function addChild(child:DisplayObject):DisplayObject {
			throw new IllegalOperationError("Error #2069: The Loader class does not implement this method.", 2069);
		}

		override public function addChildAt(child:DisplayObject, index:int):DisplayObject {
			throw new IllegalOperationError("Error #2069: The Loader class does not implement this method.", 2069);
		}

		override public function removeChild(child:DisplayObject, index:int):DisplayObject {
			throw new IllegalOperationError("Error #2069: The Loader class does not implement this method.", 2069);
		}

		override public function removeChildAt(index:int):DisplayObject {
			throw new IllegalOperationError("Error #2069: The Loader class does not implement this method.", 2069);
		}

		override public function setChildIndex(child:DisplayObject, index:int):void {
			throw new IllegalOperationError("Error #2069: The Loader class does not implement this method.", 2069);
		}
	}
}
