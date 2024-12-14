/*
   Compiled with:
   node utils/compileabc.js -p --swf StartDragTest,600,600,60 test/swfs/mouse/StartDragTest.as
*/

package 
{
    import flash.display.*;
    import flash.events.*;
    import flash.system.*;
    import flash.utils.*;
    import flash.geom.*;
    
    public class StartDragTest extends flash.display.MovieClip
    {
        private var circle:Sprite;
        public function StartDragTest()
        {
            super();
            circle = new Sprite();
            circle.graphics.beginFill(0xFFCC00);
            circle.graphics.drawCircle(0, 0, 40);
            circle.x = 100;
            circle.y = 100;
            addChild(circle);

            stage.addEventListener(MouseEvent.MOUSE_DOWN, function (e: MouseEvent): void {
                trace(circle.x + ' ' + circle.y);
                circle.startDrag();
                // in FP x and y will update in about 70-100ms
                trace(circle.x + ' ' + circle.y);
            });          

            stage.addEventListener(MouseEvent.MOUSE_UP, function (e: MouseEvent): void {
                circle.stopDrag();
                trace(circle.x + ' ' + circle.y);
            });          
        }
    }
}
