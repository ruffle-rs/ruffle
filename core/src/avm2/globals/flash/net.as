package flash.net {

    import flash.net.URLRequest;
    import flash.utils.Dictionary;
    import __ruffle__.stub_method;

    internal var _aliasToClass: Object = {};
    internal var _classToAlias: Dictionary = new Dictionary();

    public native function navigateToURL(request:URLRequest, window:String = null):void;

    public function registerClassAlias(name:String, object:Class):void {
        if (name == null) {
            throw new TypeError("Error #2007: Parameter aliasName must be non-null.", 2007);
        }
        if (object == null) {
            throw new TypeError("Error #2007: Parameter classObject must be non-null.", 2007);
        }

        this._aliasToClass[name] = object;
        this._classToAlias[object] = name;
    }

    internal function _getClassByAlias(name:String):Class {
        if (!this._aliasToClass.hasOwnProperty(name)) {
            return null;
        }

        return this._aliasToClass[name];
    }

    public function getClassByAlias(name:String):Class {
        var klass: Class = this._getClassByAlias(name);
        if (klass == null) {
            throw new ReferenceError("Error #1014: Class " + name + " could not be found.", 1014);
        }
        return klass;
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
