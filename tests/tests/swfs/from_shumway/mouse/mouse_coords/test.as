/*
   Compiled with:
   node utils/compileabc.js -p --swf MouseCoordsTest,600,600,60 test/swfs/mouse/MouseCoordsTest.as
*/

package 
{
    import flash.display.*;
    import flash.events.*;
    import flash.system.*;
    import flash.utils.*;
    import flash.geom.*;
    
    public class MouseCoordsTest extends flash.display.MovieClip
    {
        private var circle:Sprite;
        public function MouseCoordsTest()
        {
            super();
            circle = new Sprite();
            circle.graphics.beginFill(0xFFCC00);
            circle.graphics.drawCircle(0, 0, 40);
            circle.x = 100;
            circle.y = 100;
            addChild(circle);

            stage.addEventListener(MouseEvent.MOUSE_DOWN, 
                function (e: MouseEvent): void {
                trace(mouseX + ' ' + mouseY);
                trace(circle.mouseX + ' ' + circle.mouseY);
            });          
        }
    }
}
