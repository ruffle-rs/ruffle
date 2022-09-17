import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.geom.Point;


class BitmapCopyPixels {

    static function plop(mc : MovieClip, sx : Number, sy : Number,
            transp_src : Boolean, transp_dest : Boolean, transp_alpha : Boolean,
            merge: Boolean) {

        var dest:BitmapData = new BitmapData(100, 100, transp_dest, 0xBBFF0000);
        dest.fillRect(new Rectangle(20,10, 20, 80), 0x8888FF00);
        dest.fillRect(new Rectangle(50,10, 20, 80), 0x220088FF);

        var dest_img:MovieClip = mc.createEmptyMovieClip("dest_img", mc.getNextHighestDepth());
        dest_img.attachBitmap(dest, mc.getNextHighestDepth());

        dest_img._x = sx;
        dest_img._y = sy;


        var src:BitmapData = new BitmapData(80, 20, transp_src, 0x44888888);
        var src_img:MovieClip = mc.createEmptyMovieClip("src_img", mc.getNextHighestDepth());
        src_img.attachBitmap(src, mc.getNextHighestDepth());
        src.fillRect(new Rectangle(5, 5, 10, 10), 0xAA2288DD);

        src_img._x = sx + 110;
        src_img._y = sy + 20;

       	dest.copyPixels(src, new Rectangle(0,0,80,20), new Point(10, 10));


        var alpha:BitmapData = new BitmapData(40, 20, transp_alpha, 0x66884422);
        var alpha_img:MovieClip = mc.createEmptyMovieClip("alpha_img", mc.getNextHighestDepth());
        alpha_img.attachBitmap(alpha, mc.getNextHighestDepth());
        alpha.fillRect(new Rectangle(4, 8, 12, 30), 0xEE8844DD);

        alpha_img._x = sx + 110;
        alpha_img._y = sy + 60;

        dest.copyPixels(src, new Rectangle(0,0,80,20), new Point(10, 50), alpha, new Point(0,0), merge);
    }

    public static function test(mc) {
			// These values are straight alpha.
			// BitmapData internally converts to premultiplied alpha,
			// and converts back to straight alpha in 'getPixel'. This results
			// in rounding, which Ruffle currently doesn't match. These values
			// were chosen to produce the same rounded result in Flash and Ruffle.
			// FIXME - determine the correct roudning behavior to use for Ruffle,
			// and test that a 'getPixel/setPixel' round-trip always agrees between
			// Flash and Ruffle.
			var target = new BitmapData(5, 5, true, 0x45112233);
			var source = new BitmapData(5, 5, true, 0x22445566);
			var otherSource = new BitmapData(5, 5, true, 0x80aabbcc);
			
			target.copyPixels(source, new Rectangle(0, 0, 2, 2), new Point(0, 0), null, null, true);
			target.copyPixels(otherSource, new Rectangle(0, 0, 1, 1), new Point(4, 4), null, null, false);
			for (var py = 0; py < target.height; py++) {
				var line = "";
				for (var px = 0; px < target.height; px++) {
					line += target.getPixel32(px, py).toString(16) + " ";
				}
				trace(line);
			}
		
		
			var transparent = new BitmapData(1, 1, false, 0x80FFFFFF);
			var nonTransparent = new BitmapData(1, 1, false, 0x0);
			var transparentSource = new BitmapData(1, 1, true, 0xF00E0E0E);

			trace("transparentSource: " + transparentSource.getPixel32(0, 0).toString(16));

			trace("Non-transparent testing");
		
			var nonTransparentSource = new BitmapData(1, 1, false, 0x80020406);
			trace("Original pixel: " +  nonTransparent.getPixel32(0, 0).toString(16));
		
			nonTransparent.copyPixels(transparentSource, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, false);
			trace("transparent source mergeAlpha=false " + nonTransparent.getPixel32(0, 0).toString(16));
		
			nonTransparent.copyPixels(transparentSource, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, true);
			trace("transparent source mergeAlpha=true " + nonTransparent.getPixel32(0, 0).toString(16));
		
			nonTransparent.copyPixels(nonTransparentSource, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, false);
			trace("nontransparent source mergeAlpha=false " + nonTransparent.getPixel32(0, 0).toString(16));
		
			nonTransparent.copyPixels(nonTransparentSource, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, true);
			trace("nontransparent source mergeAlpha=true " + nonTransparent.getPixel32(0, 0).toString(16));
			
			trace("");
			trace("Transparent testing");
			trace("Original pixel: " +  transparent.getPixel32(0, 0).toString(16));
			
			// FIXME - enable these when Ruffle rounding is correct
		
			//transparent.copyPixels(transparentSource, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, false);
			//trace("transparent source mergeAlpha=false " + transparent.getPixel32(0, 0).toString(16));
		
			//transparent.copyPixels(transparentSource, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, true);
			//trace("transparent source mergeAlpha=true " + transparent.getPixel32(0, 0).toString(16));
		
			transparent.copyPixels(nonTransparentSource, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, false);
			trace("nontransparent source mergeAlpha=false " + transparent.getPixel32(0, 0).toString(16));
		
			transparent.copyPixels(nonTransparentSource, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, true);
			trace("nontransparent source mergeAlpha=true " + transparent.getPixel32(0, 0).toString(16));

        BitmapCopyPixels.plop(mc,  10, 20, true, true, true      , false);
        BitmapCopyPixels.plop(mc, 210, 20, true, true, false     , false);
        BitmapCopyPixels.plop(mc, 410, 20, true, false, true     , false);
        BitmapCopyPixels.plop(mc, 610, 20, true, false, false    , false);

        BitmapCopyPixels.plop(mc,  10, 220, false, true, true    , false);
        BitmapCopyPixels.plop(mc, 210, 220, false, true, false   , false);
        BitmapCopyPixels.plop(mc, 410, 220, false, false, true   , false);
        BitmapCopyPixels.plop(mc, 610, 220, false, false, false  , false);


        BitmapCopyPixels.plop(mc,  10, 520, true, true, true     , true);
        BitmapCopyPixels.plop(mc, 210, 520, true, true, false    , true);
        BitmapCopyPixels.plop(mc, 410, 520, true, false, true    , true);
        BitmapCopyPixels.plop(mc, 610, 520, true, false, false   , true);

        BitmapCopyPixels.plop(mc,  10, 720, false, true, true    , true);
        BitmapCopyPixels.plop(mc, 210, 720, false, true, false   , true);
        BitmapCopyPixels.plop(mc, 410, 720, false, false, true   , true);
        BitmapCopyPixels.plop(mc, 610, 720, false, false, false  , true);
    }
}
