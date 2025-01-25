package {
import flash.display.*;
import flash.text.*;
import flash.geom.*;

[SWF(width="256", height="128")]
public class Test extends Sprite {
    [Embed(source="rgba-random.png")]
    private static var RgbaRandom:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        var randomBitmapData:BitmapData = (new RgbaRandom() as Bitmap).bitmapData;

        var basicBitmapData:BitmapData = new BitmapData(128, 128, true, 0xFF000000);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(46, 21, 32, 32),
            new Point(0, 0),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(168, 81, 32, 32),
            new Point(32, 0),
            BitmapDataChannel.GREEN,
            BitmapDataChannel.GREEN);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(2, 204, 32, 32),
            new Point(64, 0),
            BitmapDataChannel.BLUE,
            BitmapDataChannel.BLUE);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(105, 182, 32, 32),
            new Point(96, 0),
            BitmapDataChannel.ALPHA,
            BitmapDataChannel.ALPHA);

        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(46, 121, 32, 32),
            new Point(0, 32),
            BitmapDataChannel.GREEN,
            BitmapDataChannel.RED);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(65, 81, 32, 32),
            new Point(32, 32),
            BitmapDataChannel.BLUE,
            BitmapDataChannel.GREEN);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(24, 204, 32, 32),
            new Point(64, 32),
            BitmapDataChannel.ALPHA,
            BitmapDataChannel.BLUE);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(105, 143, 32, 32),
            new Point(96, 32),
            BitmapDataChannel.RED,
            BitmapDataChannel.ALPHA);

        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(111, 2, 32, 32),
            new Point(0, 64),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(111, 2, 32, 32),
            new Point(32, 64),
            BitmapDataChannel.GREEN,
            BitmapDataChannel.RED);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(111, 2, 32, 32),
            new Point(64, 64),
            BitmapDataChannel.BLUE,
            BitmapDataChannel.RED);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(111, 2, 32, 32),
            new Point(96, 64),
            BitmapDataChannel.ALPHA,
            BitmapDataChannel.RED);

        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(12, 92, 32, 32),
            new Point(0, 64),
            BitmapDataChannel.RED,
            BitmapDataChannel.GREEN);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(12, 92, 32, 32),
            new Point(32, 64),
            BitmapDataChannel.GREEN,
            BitmapDataChannel.GREEN);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(12, 92, 32, 32),
            new Point(64, 64),
            BitmapDataChannel.BLUE,
            BitmapDataChannel.GREEN);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(12, 92, 32, 32),
            new Point(96, 64),
            BitmapDataChannel.ALPHA,
            BitmapDataChannel.GREEN);

        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(0, 64),
            BitmapDataChannel.RED,
            BitmapDataChannel.BLUE);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(32, 64),
            BitmapDataChannel.GREEN,
            BitmapDataChannel.BLUE);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(64, 64),
            BitmapDataChannel.BLUE,
            BitmapDataChannel.BLUE);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(96, 64),
            BitmapDataChannel.ALPHA,
            BitmapDataChannel.BLUE);

        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(0, 96),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(32, 96),
            BitmapDataChannel.GREEN,
            BitmapDataChannel.GREEN);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(64, 96),
            BitmapDataChannel.BLUE,
            BitmapDataChannel.BLUE);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(96, 96),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);

        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(0, 96),
            BitmapDataChannel.GREEN,
            BitmapDataChannel.ALPHA);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(32, 96),
            BitmapDataChannel.BLUE,
            BitmapDataChannel.ALPHA);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(64, 96),
            BitmapDataChannel.RED,
            BitmapDataChannel.ALPHA);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(96, 96),
            BitmapDataChannel.ALPHA,
            BitmapDataChannel.ALPHA);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(96, 96),
            BitmapDataChannel.RED,
            BitmapDataChannel.ALPHA);
        basicBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(42, 69, 32, 32),
            new Point(96, 96),
            BitmapDataChannel.GREEN,
            BitmapDataChannel.ALPHA);

        var basicBitmap:Bitmap = new Bitmap(basicBitmapData);
        basicBitmap.x = 0;
        basicBitmap.y = 0;
        addChild(basicBitmap);

        var oobBitmapData:BitmapData = new BitmapData(128, 128, true, 0xFF0000FF);
        // source out of bounds
        oobBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(240, 240, 32, 32),
            new Point(0, 0),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);
        oobBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(-10, -10, 32, 32),
            new Point(32, 0),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);

        // destination out of bounds
        oobBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(0, 0, 32, 32),
            new Point(110, 110),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);
        oobBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(0, 0, 32, 32),
            new Point(-10, 32),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);

        // both out of bounds
        oobBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(-10, 0, 32, 64),
            new Point(100, 32),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);

        // no common rect
        oobBitmapData.copyChannel(
            randomBitmapData,
            new Rectangle(-10, -10, 32, 32),
            new Point(120, 120),
            BitmapDataChannel.RED,
            BitmapDataChannel.RED);

        var oobBitmap:Bitmap = new Bitmap(oobBitmapData);
        oobBitmap.x = 128;
        oobBitmap.y = 0;
        addChild(oobBitmap);
    }
}
}
