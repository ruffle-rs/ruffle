package flash.net {

    import flash.net.URLRequest;
    import __ruffle__.stub_method;

    public native function navigateToURL(request:URLRequest, window:String = null):void;

    public native function registerClassAlias(name:String, object:Class):void;
    public native function getClassByAlias(name:String):Class;

    public function sendToURL(request:URLRequest):void {
        stub_method("flash.net", "sendToURL");
    }
}
