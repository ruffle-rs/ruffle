// compiled with mxmlc

package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test(){
            graphics.beginFill(0xFFCC00);
            // top left: normal
            graphics.drawRoundRect(10, 10, 100, 100, 70, 30);
            // top right: clamping
            graphics.drawRoundRect(150, 10, 100, 100, 100, 100);

            // middle left: complex, normal
            graphics.drawRoundRectComplex(10, 150, 100, 100,
                20, 30, 40, 50);

            // middle right: complex, clamping
            graphics.drawRoundRectComplex(150, 150, 100, 100,
                80, 60, 50, 30);

            // bottom left: circle
            graphics.drawCircle(80, 300, 50);
            // bottom right: ellipse
            graphics.drawEllipse(150, 300, 100, 50);
            graphics.endFill();
        }
    }
}
