package  {
    
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.events.FocusEvent;
    import flash.net.URLRequest;
    import flash.text.TextField;
    import flash.text.TextFormat;
    import flash.display.Sprite;
    import flash.display.Shape;
    import flash.display.SimpleButton;

    
    public class Test extends MovieClip {
        
        
        var loader:Loader = new Loader();
        
        public function Test() {
            super();
            stage.addEventListener("mouseFocusChange", this.onMouseFocusChange);

            loader.load(new URLRequest("avm1_child.swf"));
            loader.y = 250
            addChild(loader);
            
            var txt1 = new TextField();
            txt1.name = "txt1";
            txt1.type = "input";
            txt1.width = 200;
            txt1.height = 50;
            addChild(txt1);
            txt1.addEventListener("focusIn", this.onFocusIn);
            txt1.addEventListener("focusOut", this.onFocusOut);
            
            var rectTab = new Sprite();
            rectTab.name = "rectTabEnabled";
            rectTab.graphics.beginFill(0xFF0000);
            rectTab.graphics.drawRect(0, 0, 200, 50);
            rectTab.graphics.endFill();
            rectTab.y = 50;
            rectTab.tabEnabled = true;
            addChild(rectTab);
            rectTab.addEventListener("focusIn", this.onFocusIn);
            rectTab.addEventListener("focusOut", this.onFocusOut);
            
            var rectBtn = new Sprite();
            rectBtn.name = "rectButtonMode";
            rectBtn.graphics.beginFill(0x0000FF);
            rectBtn.graphics.drawRect(0, 0, 200, 50);
            rectBtn.graphics.endFill();
            rectBtn.y = 100;
            rectBtn.buttonMode = true;
            addChild(rectBtn);
            rectBtn.addEventListener("focusIn", this.onFocusIn);
            rectBtn.addEventListener("focusOut", this.onFocusOut);
            
            var rectNorm = new Sprite();
            rectNorm.name = "rectNormal";
            rectNorm.graphics.beginFill(0x00FF00);
            rectNorm.graphics.drawRect(0, 0, 200, 50);
            rectNorm.graphics.endFill();
            rectNorm.y = 150;
            addChild(rectNorm);
            
            var btnShape = new Shape();
            btnShape.graphics.beginFill(0xFFFF00);
            btnShape.graphics.drawRect(0, 0, 200, 50);
            btnShape.graphics.endFill();
            
            var btn = new SimpleButton(btnShape, btnShape, btnShape, btnShape);
            btn.name = "simpleBtn";
            btn.y = 200;
            addChild(btn);
            btn.addEventListener("focusIn", this.onFocusIn);
            btn.addEventListener("focusOut", this.onFocusOut);
        }

        public function onFocusIn(e:FocusEvent) {
            trace("focusIn", e.target.name, e.relatedObject);
        }
        
        public function onFocusOut(e:FocusEvent) {
            trace("focusOut", e.target.name, e.relatedObject);
        }

        public function onMouseFocusChange(e:FocusEvent) {
            if (e.relatedObject != null) {
                trace("mouse " + e.relatedObject + " (" + e.relatedObject.name + ")");
            } else {
                trace("mouse " + e.relatedObject);
            }
        }
    }
    
}