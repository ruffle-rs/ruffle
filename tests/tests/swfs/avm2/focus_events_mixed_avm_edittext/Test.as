package  {
    
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.events.Event;
    import flash.events.FocusEvent;
    import flash.net.URLRequest;

    
    public class Test extends MovieClip {
        
        
        public function Test() {
            super();
            stage.addEventListener("mouseFocusChange", this.onMouseFocusChange);
            stage.addEventListener("keyFocusChange", this.onKeyFocusChange);
            
            this.as3Input1.addEventListener("focusIn", this.onFocusIn);
            this.as3Input1.addEventListener("focusOut", this.onFocusOut);
            this.as3Input2.addEventListener("focusIn", this.onFocusIn);
            this.as3Input2.addEventListener("focusOut", this.onFocusOut);
            this.buttonMc.addEventListener("focusIn", this.onFocusIn);
            this.buttonMc.addEventListener("focusOut", this.onFocusOut);

            var loader:Loader = new Loader();
            loader.load(new URLRequest("avm1.swf"));
            addChild(loader);
            
            this.buttonMc.buttonMode = true;
        }
        
        public function onFocusIn(e:FocusEvent):void {
            if (e.relatedObject != null) {
                trace("focusIn -", "Target:", e.target.name, "relatedObject:", e.relatedObject.name);
            } else {
                trace("focusIn -", "Target:", e.target.name, "relatedObject:", e.relatedObject);
            }
        }
        
        public function onFocusOut(e:FocusEvent):void {
            if (e.relatedObject != null) {
                trace("focusOut -", "Target:", e.target.name, "relatedObject:", e.relatedObject.name);
            } else {
                trace("focusOut -", "Target:", e.target.name, "relatedObject:", e.relatedObject);
            }
        }
        
        public function onMouseFocusChange(e:FocusEvent) {
            trace("mouseFocusChange: " + e.relatedObject);
        }
        
        public function onKeyFocusChange(e:FocusEvent) {
            trace("keyFocusChange: " + e.relatedObject);
        }
    }
    
}