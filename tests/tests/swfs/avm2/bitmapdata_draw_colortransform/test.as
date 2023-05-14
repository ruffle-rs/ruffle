package {
    import flash.display.DisplayObjectContainer;
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.text.TextField;
    import flash.display.Shape;
    import flash.display.Stage;
    import flash.display.Sprite;
    import flash.geom.Matrix;
    import flash.geom.Rectangle;
    import flash.geom.ColorTransform;

    public class test extends Sprite {
        public function test() {

            var sourceBitmapData:BitmapData = new BitmapData(100,100, false, 0xFFFF0000);

            var colorTransform:ColorTransform = new ColorTransform(1, 0.5, 0.1, 0.2, -100, 50, 10, 30);


            var destBitmapData1:BitmapData = new BitmapData(100,100, false, 0xFF1199DD);
            destBitmapData1.draw(sourceBitmapData, null, colorTransform);
            var destBitmap1:Bitmap = new Bitmap(destBitmapData1);
            destBitmap1.x = 0;
            this.addChild(destBitmap1);


            var destBitmapData2:BitmapData = new BitmapData(100,100, false, 0xFF1199DD);
            destBitmapData2.draw(destBitmapData2, null, colorTransform);
            var destBitmap2:Bitmap = new Bitmap(destBitmapData2);
            destBitmap2.x = 100;
            this.addChild(destBitmap2);

        }
    }
}
