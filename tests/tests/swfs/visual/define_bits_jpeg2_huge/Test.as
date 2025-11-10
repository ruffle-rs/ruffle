package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source="image_width.png")]
    private var Image1:Class;
    [Embed(source="image_height.png")]
    private var Image2:Class;
    [Embed(source="image_width.jpeg")]
    private var Image3:Class;
    [Embed(source="image_height.jpeg")]
    private var Image4:Class;

    public function Test() {
        trace("Loaded");
        var image1:Bitmap = new Image1();
        var image2:Bitmap = new Image2();
        var image3:Bitmap = new Image3();
        var image4:Bitmap = new Image4();

        trace("Image 1");
        trace(image1.bitmapData.width);
        trace(image1.bitmapData.height);
        printPixels(image1.bitmapData);

        trace("Image 2");
        trace(image2.bitmapData.width);
        trace(image2.bitmapData.height);
        printPixels(image2.bitmapData);

        trace("Image 3");
        trace(image3.bitmapData.width);
        trace(image3.bitmapData.height);
        // This throws "Error #2015: Invalid BitmapData." in Flash Player, so
        // let's ignore it as that's not the point of the test.
        // printPixels(image3.bitmapData);

        trace("Image 4");
        trace(image4.bitmapData.width);
        trace(image4.bitmapData.height);
        printPixels(image4.bitmapData);
    }

    private function printPixels(bd: BitmapData) {
        try {
            trace(bd.getPixel32(0, 0));
            trace(bd.getPixel32(bd.width - 1, bd.height - 1));
        } catch (e:Error) {
            trace("Error:");
            trace(e);
        }
    }
}
}
