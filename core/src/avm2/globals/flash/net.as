package flash.net {

    import flash.net.URLRequest;
    import flash.utils.Dictionary;
    import __ruffle__.stub_method;
    
    internal var _aliasToClass: Object = {};
    internal var _classToAlias: Dictionary = new Dictionary();

    public native function navigateToURL(request:URLRequest, window:String = null):void;

    public function registerClassAlias(name:String, object:Class):void {
        if (name == null || object == null) {
            throw new TypeError("FIXME - what error should I use");
        }
        this._aliasToClass[name] = object;
        this._classToAlias[object] = name;
    }
    
    public function getClassByAlias(name:String):Class {
        if (this._aliasToClass[name]) {
            return this._aliasToClass[name];
        } else {
            return null;
        }
    }

    internal function _getAliasByClass(object:Class):String {
        if (this._classToAlias[object]) {
            return this._classToAlias[object];
        } else {
            return null;
        }
    }

    public function sendToURL(request:URLRequest):void {
        stub_method("flash.net", "sendToURL");
    }
}
