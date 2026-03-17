package  {
    
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.events.Event;
    import flash.events.FocusEvent;
    import flash.net.URLRequest;

    
    public class Test extends MovieClip {
        
        
        var loader:Loader = new Loader();
        
        public function Test() {
            super();
            stage.addEventListener("mouseFocusChange", this.onMouseFocusChange);
            stage.addEventListener("keyFocusChange", this.onKeyFocusChange);

            this.loader.contentLoaderInfo.addEventListener("complete", this.onChildLoaded);
            this.loader.load(new URLRequest("avm1.swf"));
            
            this.buttonMc.buttonMode = true;
        }
        
        public function onChildLoaded(e:Event):void {
            addChild(this.loader);
        }
        
        public function onMouseFocusChange(e:FocusEvent) {
            trace("mouse " + e.relatedObject);
        }
        
        public function onKeyFocusChange(e:FocusEvent) {
            trace("key " + e.relatedObject);
        }
    }
    
}