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

    static function main(mc) {
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