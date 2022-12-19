package flash.net {

    import flash.net.URLRequest;
    import __ruffle__.log_warn;

    public native function navigateToURL(request:URLRequest, window:String = null):void;

    public function registerClassAlias(a:String, b:Object):void {
        log_warn("flash.net.registerClassAlias - is not implemented");
    }
}