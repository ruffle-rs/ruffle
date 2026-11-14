package flash.display {
    import flash.net.URLRequest;
    import flash.system.LoaderContext;

    [API("688")]
    public class AVLoader extends Loader {
        public function AVLoader() {
            super();
        }

        override public function load(request:URLRequest, ctx:LoaderContext = null):void {
            super.load(request, ctx);
        }
    }
}
