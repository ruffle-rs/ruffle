package flash.net {
    import flash.net.URLRequest;

    public native function navigateToURL(request:URLRequest, window:String = null):void;

    public native function registerClassAlias(name:String, object:Class):void;
    public native function getClassByAlias(name:String):Class;

    public native function sendToURL(request:URLRequest):void;
}
