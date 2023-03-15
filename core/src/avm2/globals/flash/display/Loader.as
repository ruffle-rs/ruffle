package flash.display {
	public class Loader extends DisplayObjectContainer {
		import flash.display.LoaderInfo;
		import flash.display.DisplayObject;
		import flash.errors.IllegalOperationError;
		import flash.system.LoaderContext;
		import flash.utils.ByteArray;
		import flash.net.URLRequest;
   		import __ruffle__.stub_method;

		internal var _contentLoaderInfo: LoaderInfo;

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
			stub_method("flash.display.Loader", "unload");
			// Content seems to prefer an error here, over an empty implementation.
			// https://github.com/ruffle-rs/ruffle/pull/8909
			throw new Error("flash.display.Loader.unload - not yet implemented");
		}

		public function unloadAndStop(gc:Boolean = true):void {
			stub_method("flash.display.Loader", "unloadAndStop");
		}
		
		public function close():void {
			stub_method("flash.display.Loader", "close");
		}

		override public function addChild(child:DisplayObject):DisplayObject {
			throw new IllegalOperationError("Error #2069: The Loader class does not implement this method.", 2069);
		}

		override public function addChildAt(child:DisplayObject, index:int):DisplayObject {
			throw new IllegalOperationError("Error #2069: The Loader class does not implement this method.", 2069);
		}

		override public function removeChild(child:DisplayObject):DisplayObject {
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
