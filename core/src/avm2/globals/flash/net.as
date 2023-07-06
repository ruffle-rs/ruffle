package flash.net {

    import flash.net.URLRequest;
    import __ruffle__.stub_method;
    
    internal var _classLookups:Object = {};

    public native function navigateToURL(request:URLRequest, window:String = null):void;

    public function registerClassAlias(name:String, object:Class):void {
        stub_method("flash.net", "registerClassAlias");
        this._classLookups[name] = object;
    }
    
    public function getClassByAlias(name:String):Class {
        if (this._classLookups[name]) {
            return this._classLookups[name];
        } else {
            return null;
        }
    }

    public function sendToURL(request:URLRequest):void {
        stub_method("flash.net", "sendToURL");
    }
}
