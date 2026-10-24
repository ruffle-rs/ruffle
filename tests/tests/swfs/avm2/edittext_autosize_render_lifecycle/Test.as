package {
    import flash.display.Shape;
    import flash.display.SimpleButton;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.geom.Rectangle;
    import flash.text.TextField;
    [SWF(width="640", height="480", frameRate="24")]
    public class Test extends Sprite {
        private var frame:int = 0;
        private var cases:Array = [];
        public function Test() {
            stage.scaleMode = "noScale";
            stage.align = "TL";
            var background:Shape = new Shape();
            background.graphics.beginFill(0xEEEEEE);
            background.graphics.drawRect(-100, -100, 840, 680);
            background.graphics.endFill();
            addChild(background);
            make("enter-to-exit", "enter", "exit");
            make("enter-to-render", "enter", "render");
            make("enter-to-next-enter", "enter", "next");
            make("exit-to-render", "exit", "render");
            make("exit-to-next-enter", "exit", "next");
            make("render-to-next-enter", "render", "next");
            make("hidden-parent", "enter", "next", "hidden");
            make("scrollrect", "enter", "next", "clip");
            make("offstage", "enter", "next", "offstage");
            make("button-up", "enter", "next", "button");
            make("hidden-button", "enter", "next", "hidden-button");
            make("inactive-button", "enter", "next", "inactive-button");
            stage.addEventListener(Event.ENTER_FRAME, function(event:Event):void {
                frame++;
                stage.invalidate();
                handle("enter");
            });
            stage.addEventListener(Event.EXIT_FRAME, function(event:Event):void { handle("exit"); });
            stage.addEventListener(Event.RENDER, function(event:Event):void { handle("render"); });
        }
        private function make(name:String, reset:String, observe:String, mode:String = "normal"):void {
            var parent:Sprite = new Sprite();
            var text:TextField = new TextField();
            text.visible = false;
            parent.addChild(text);
            if (mode == "hidden") parent.visible = false;
            if (mode == "clip") parent.scrollRect = new Rectangle(0, 0, 20, 20);
            if (mode.indexOf("button") != -1) {
                var up:Sprite = mode == "inactive-button" ? new Sprite() : parent;
                var hit:Sprite = new Sprite();
                hit.graphics.beginFill(0);
                hit.graphics.drawRect(400, 300, 20, 20);
                hit.graphics.endFill();
                var button:SimpleButton = new SimpleButton(up, parent, parent, hit);
                button.visible = mode != "hidden-button";
                addChild(button);
            } else if (mode != "offstage") addChild(parent);
            cases.push({name:name, reset:reset, observe:observe, text:text, done:false});
        }
        private function handle(phase:String):void {
            for each (var c:Object in cases) {
                if (frame == 2 && c.reset == phase) {
                    c.text.autoSize = "none";
                    c.text.wordWrap = false;
                    c.text.width = 200;
                    c.text.x = 0;
                    c.text.autoSize = "center";
                }
                if (!c.done && ((frame == 2 && c.observe == phase) ||
                    (frame == 3 && phase == "enter" && c.observe == "next"))) {
                    c.done = true;
                    c.text.wordWrap = true;
                    trace("CASE " + c.name + " width=" + c.text.width + " x=" + c.text.x);
                }
            }
        }
    }
}
