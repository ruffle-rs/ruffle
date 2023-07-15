import flash.display.BitmapData;
import flash.geom.ColorTransform;

addTestBitmap(0, 1, 1, 1, 1, 0, 0, 0, 0);
addTestBitmap(1, 2, 1, 1, 1, 0, 0, 0, 0);
addTestBitmap(2, 1, 2, 1, 1, 0, 0, 0, 0);
addTestBitmap(3, 1, 1, 2, 1, 0, 0, 0, 0);

// Flash bug: Transform with only alpha multiplier > 1 has no effect!
addTestBitmap(4, 1, 1, 1, 2, 0, 0, 0, 0); 

// But if any other property is non-default, it does have an effect.
addTestBitmap(5, 2, 1, 1, 2, 0, 0, 0, 0); 

addTestBitmap(6, 0.5, 1, 1, 1, 0, 0, 0, 0);
addTestBitmap(7, 1, 0.5, 1, 1, 0, 0, 0, 0);
addTestBitmap(8, 1, 1, 0.5, 1, 0, 0, 0, 0);
addTestBitmap(9, 1, 1, 1, 0.5, 0, 0, 0, 0);
addTestBitmap(10, 1, 1, 1, 1, 50, 0, 0, 0);
addTestBitmap(11, 1, 1, 1, 1, 0, 50, 0, 0);
addTestBitmap(12, 1, 1, 1, 1, 0, 0, 50, 0);

// Additive alpha should not affect pixels with 0 alpha.
addTestBitmap(13, 1, 1, 1, 1, 0, 0, 0, 50);

addTestBitmap(14, 1, 1, 1, 1, -50, 0, 0, 0);
addTestBitmap(15, 1, 1, 1, 1, 0, -50, 0, 0);
addTestBitmap(16, 1, 1, 1, 1, 0, 0, -50, 0);
addTestBitmap(17, 1, 1, 1, 1, 0, 0, 0, -50);

// Colors should saturate
addTestBitmap(18, 1, 1, 1, 1, 32764, -32764, 0, 0);

// But intermediate calculations should have 16-bit precision, saturation only on final color
addTestBitmap(19, 127.99609375, 1, 1, 1, -16255, 0, 0, 0);

addTestBitmap(20, 0.5, 1.3, 0.2, 0.9, 5, 33, -44, -12);

function addTestBitmap(y:Number, rMult:Number, gMult:Number, bMult:Number, aMult:Number, rAdd:Number, gAdd:Number, bAdd:Number, aAdd:Number)
{
    var bData:BitmapData = BitmapData.loadBitmap("TestBitmapData");
    var ct:ColorTransform = new ColorTransform(rMult, gMult, bMult, aMult, rAdd, gAdd, bAdd, aAdd);
    bData.colorTransform(bData.rectangle, ct);
    var bitmap = createEmptyMovieClip("bitmap" + y, y);
    bitmap.attachBitmap(bData, 0);
    bitmap._y = y * 8;
}
