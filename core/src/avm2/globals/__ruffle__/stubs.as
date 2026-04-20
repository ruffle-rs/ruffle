package __ruffle__ {
    // Mark the method where this function is invoked as stubbed.
    public native function stub_method(...rest):void;

    // Mark the getter where this function is invoked as stubbed.
    public native function stub_getter(...rest):void;

    // Mark the setter where this function is invoked as stubbed.
    public native function stub_setter(...rest):void;

    // Mark the constructor where this function is invoked as stubbed.
    public native function stub_constructor(...rest):void;

    // Note: the following function is not related to stubbing.

    // Produce a regular warning in Ruffle logs.
    public native function log_warn(...rest):void;
}
