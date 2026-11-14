package flash.net {
    import flash.events.DataEvent;
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.events.IOErrorEvent;
    import flash.events.PermissionEvent;
    import flash.events.ProgressEvent;
    import flash.events.SecurityErrorEvent;
    import flash.permissions.PermissionStatus;
    import flash.utils.ByteArray;

    [Ruffle(InstanceAllocator)]
    public class FileReference extends EventDispatcher {
        private var _activeLoader:URLLoader;

        public function FileReference() {}

        public native function get creationDate():Date;

        public function get creator():String {
            // This was macOS (pre OS X) only. (Deprecated)
            return null;
        }

        public native function get data():ByteArray;

        // AIR 1.0
        [API("661")]
        public function get extension():String {
            // The file extension, excluding the dot.
            return this.type ? this.type.slice(1) : null;
        }

        public native function get modificationDate():Date;

        public native function get name():String;

        public native function get size():Number;

        // File extension, including the dot. (Deprecated)
        public native function get type():String;

        public native function browse(typeFilter:Array = null):Boolean;

        public function cancel():void {
            if (this._activeLoader != null) {
                try {
                    this._activeLoader.close();
                } catch (e:Error) {
                }
                this._activeLoader = null;
            }
        }

        public function download(request:URLRequest, defaultFileName:String = null):void {
            if (request == null) {
                Error.throwError(TypeError, 2007, "request");
            }
            this.cancel();

            var fileName:String = defaultFileName;
            if (fileName == null || fileName.length == 0) {
                var url:String = request.url || "";
                var qIdx:int = url.indexOf("?");
                if (qIdx >= 0) {
                    url = url.substring(0, qIdx);
                }
                var slashIdx:int = url.lastIndexOf("/");
                fileName = slashIdx >= 0 ? url.substring(slashIdx + 1) : url;
            }

            var self:FileReference = this;
            var loader:URLLoader = new URLLoader();
            loader.dataFormat = URLLoaderDataFormat.BINARY;
            this._activeLoader = loader;

            loader.addEventListener(Event.OPEN, function(e:Event):void {
                self.dispatchEvent(new Event(Event.OPEN));
            });
            loader.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
                self.dispatchEvent(e.clone());
            });
            loader.addEventListener(IOErrorEvent.IO_ERROR, function(e:IOErrorEvent):void {
                self._activeLoader = null;
                self.dispatchEvent(e.clone());
            });
            loader.addEventListener(SecurityErrorEvent.SECURITY_ERROR, function(e:SecurityErrorEvent):void {
                self._activeLoader = null;
                self.dispatchEvent(e.clone());
            });
            loader.addEventListener(Event.COMPLETE, function(e:Event):void {
                self._activeLoader = null;
                self.save(loader.data, fileName);
            });

            loader.load(request);
        }

        public native function load():void;

        [API("681")]
        public function requestPermission():void {
            this.dispatchEvent(new PermissionEvent(PermissionEvent.PERMISSION_STATUS, false, false, PermissionStatus.GRANTED));
        }

        public function save(data:*, defaultFileName:String = null):void {
            if (data == null) {
                // Yes, this is the error that Flash throws
                throw new ArgumentError("data");
            }

            // Convert the data to a ByteArray
            var byteArray:ByteArray = new ByteArray();
            if (data is ByteArray) {
                byteArray.writeBytes(data);
            } else if (data is XML) {
                byteArray.writeUTFBytes(data.toXMLString());
            } else {
                byteArray.writeUTFBytes(data);
            }

            if (defaultFileName == null) {
                defaultFileName = "";
            }

            this.saveInternal(byteArray, defaultFileName);
        }

        // NOTE: The bytearray passed to this function will be cleared
        private native function saveInternal(data:ByteArray, defaultFileName:String):void;

        public function upload(
            request:URLRequest,
            uploadDataFieldName:String = "Filedata",
            testUpload:Boolean = false
        ):void {
            if (request == null) {
                Error.throwError(TypeError, 2007, "request");
            }
            this.cancel();

            var self:FileReference = this;
            var loader:URLLoader = new URLLoader();
            this._activeLoader = loader;

            var uploadReq:URLRequest = new URLRequest(request.url);
            uploadReq.method = URLRequestMethod.POST;
            uploadReq.data = this.data != null ? this.data : new ByteArray();

            loader.addEventListener(Event.OPEN, function(e:Event):void {
                self.dispatchEvent(new Event(Event.OPEN));
            });
            loader.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
                self.dispatchEvent(e.clone());
            });
            loader.addEventListener(IOErrorEvent.IO_ERROR, function(e:IOErrorEvent):void {
                self._activeLoader = null;
                self.dispatchEvent(e.clone());
            });
            loader.addEventListener(SecurityErrorEvent.SECURITY_ERROR, function(e:SecurityErrorEvent):void {
                self._activeLoader = null;
                self.dispatchEvent(e.clone());
            });
            loader.addEventListener(Event.COMPLETE, function(e:Event):void {
                self._activeLoader = null;
                self.dispatchEvent(new Event(Event.COMPLETE));
                self.dispatchEvent(new DataEvent(DataEvent.UPLOAD_COMPLETE_DATA, false, false, String(loader.data)));
            });

            loader.load(uploadReq);
        }

        [API("681")]
        public function uploadUnencoded(request:URLRequest):void {
            this.upload(request);
        }
    }
}
