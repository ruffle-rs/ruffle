package flash.display {

	[Ruffle(InstanceAllocator)]
	public class Loader extends DisplayObjectContainer {
		import flash.display.LoaderInfo;
		import flash.display.DisplayObject;
		import flash.errors.IllegalOperationError;
		import flash.system.LoaderContext;
		import flash.utils.ByteArray;
		import flash.net.URLRequest;
		import flash.events.UncaughtErrorEvents;
   		import __ruffle__.stub_method;

		internal var _contentLoaderInfo: LoaderInfo;

		public function get contentLoaderInfo():LoaderInfo {
			return this._contentLoaderInfo;
		}

		public function get content():DisplayObject {
			return this._contentLoaderInfo.content;
		}

		public native function load(request: URLRequest, context: LoaderContext = null):void;

		public native function loadBytes(data: ByteArray, context: LoaderContext = null):void;
		
		public native function unload():void;

		public function unloadAndStop(gc:Boolean = true):void {
			stub_method("flash.display.Loader", "unloadAndStop");
			this.unload();
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

		public function get uncaughtErrorEvents():UncaughtErrorEvents {
			return this.contentLoaderInfo.uncaughtErrorEvents;
		}
	}
}
