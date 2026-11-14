package flash.desktop {
    import flash.system.System;
    import flash.utils.ByteArray;

    public class Clipboard {
        private static var _generalClipboard:Clipboard = new Clipboard();

        private var _data:Object = {};
        private var _handlers:Object = {};
        private var _serializable:Object = {};

        public static function get generalClipboard():Clipboard {
            return Clipboard._generalClipboard;
        }

        public function Clipboard() {
        }

        public function get formats():Array {
            var result:Array = [];
            var key:String;
            for (key in this._data) {
                result.push(key);
            }
            for (key in this._handlers) {
                if (!(key in this._data)) {
                    result.push(key);
                }
            }
            return result;
        }

        public function clear():void {
            this._data = {};
            this._handlers = {};
            this._serializable = {};
        }

        public function clearData(format:String):void {
            if (format == null) {
                Error.throwError(TypeError, 2007, "format");
            }
            delete this._data[format];
            delete this._handlers[format];
            delete this._serializable[format];
        }

        public function getData(
            format:String,
            transferMode:String = ClipboardTransferMode.ORIGINAL_PREFERRED
        ):Object {
            if (format == null) {
                Error.throwError(TypeError, 2007, "format");
            }

            var value:Object = null;
            if (format in this._handlers) {
                var handler:Function = this._handlers[format];
                value = handler();
            } else if (format in this._data) {
                value = this._data[format];
            } else {
                return null;
            }

            if (value != null && transferMode == ClipboardTransferMode.CLONE_ONLY) {
                if (!this._serializable[format]) {
                    return null;
                }
                var bytes:ByteArray = new ByteArray();
                bytes.writeObject(value);
                bytes.position = 0;
                return bytes.readObject();
            }

            return value;
        }

        public function hasFormat(format:String):Boolean {
            if (format == null) {
                Error.throwError(TypeError, 2007, "format");
            }
            return (format in this._data) || (format in this._handlers);
        }

        public function setData(format:String, data:Object, serializable:Boolean = true):Boolean {
            if (format == null) {
                Error.throwError(TypeError, 2007, "format");
            }
            if (data == null) {
                Error.throwError(TypeError, 2007, "data");
            }

            delete this._handlers[format];
            this._data[format] = data;
            this._serializable[format] = serializable;

            if (format == ClipboardFormats.TEXT_FORMAT) {
                System.setClipboard(String(data));
            }
            return true;
        }

        public function setDataHandler(format:String, handler:Function, serializable:Boolean = true):Boolean {
            if (format == null) {
                Error.throwError(TypeError, 2007, "format");
            }
            if (handler == null) {
                Error.throwError(TypeError, 2007, "handler");
            }

            delete this._data[format];
            this._handlers[format] = handler;
            this._serializable[format] = serializable;
            return true;
        }
    }
}
