package __ruffle__ {
    // Mark the method where this function is invoked as stubbed.
    public native function stub_method(className:String, methodName:String, specifics:String = null):void;

    // Mark the getter where this function is invoked as stubbed.
    public native function stub_getter(className:String, propertyName:String):void;

    // Mark the setter where this function is invoked as stubbed.
    public native function stub_setter(className:String, propertyName:String):void;

    // Mark the constructor where this function is invoked as stubbed.
    public native function stub_constructor(className:String, specifics:String = null):void;

    // Note: the following function is not related to stubbing.

    // Produce a regular warning in Ruffle logs.
    public native function log_warn(...rest):void;
}
