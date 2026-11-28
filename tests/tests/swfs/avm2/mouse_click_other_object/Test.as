// Pressing the mouse over one object and releasing it over another one
// doesn't dispatch a click, even if both objects look alike (same kind of
// object, at the same depth in their parents, from the same movie).
package {
    import flash.display.Sprite;
    import flash.events.MouseEvent;

    public class Test extends Sprite {
        public function Test() {
            addBox("left", 0);
            addBox("right", 200);

            stage.addEventListener(MouseEvent.CLICK, function(event:MouseEvent):void {
                trace("stage: click on " + event.target.name);
            });
        }

        private function addBox(name:String, x:Number):void {
            var parent:Sprite = new Sprite();
            parent.name = name + "Parent";
            parent.x = x;
            addChild(parent);

            var box:Sprite = new Sprite();
            box.name = name;
            box.graphics.beginFill(0xFF0000);
            box.graphics.drawRect(0, 0, 100, 100);
            box.graphics.endFill();
            parent.addChild(box);

            for each (var type:String in [MouseEvent.MOUSE_DOWN, MouseEvent.MOUSE_UP, MouseEvent.CLICK]) {
                box.addEventListener(type, function(event:MouseEvent):void {
                    trace(event.currentTarget.name + ": " + event.type);
                });
            }
        }
    }
}
