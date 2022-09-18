package flash.display {
    import flash.events.ErrorEvent;
    import flash.events.EventDispatcher;
    import flash.display3D.Context3D;
    import flash.utils.setTimeout;
    
    [Ruffle(InstanceAllocator)]
    public class Stage3D extends EventDispatcher {
        
        public native function get context3D():Context3D;
        private native function requestContext3D_internal(context3DRenderMode:String, profile:String):void;

        public function requestContext3D(context3DRenderMode:String = "auto", profile:String = "baseline"):void {
            // Several SWFS (the examples from the Context3D documentation, and the Starling framework)
            // rely on the `context3DCreate` being fired asynchronously - they initialize variables
            // after the call to `requestContext3D`, and then use those variables in the event handler.
            // Currently, we create a `Context3D` synchronously, so we need to delay the event dispatch
            var stage3d = this;
            setTimeout(function() {
                stage3d.requestContext3D_internal(context3DRenderMode, profile);
            }, 0);
        }

        // FIXME - actually implement this
        public var x:Number;
        public var y:Number;
    }
}