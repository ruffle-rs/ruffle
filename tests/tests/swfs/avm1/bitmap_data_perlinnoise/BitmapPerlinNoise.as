import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.geom.Point;

class BitmapPerlinNoise {

    static function plop(mc : MovieClip, sx : Number, sy : Number,
        transp : Boolean,
        bx : Number, by : Number,
        numOctaves : Number,
        seed : Number,
        stitch: Boolean,
        fractalNoise: Boolean,
        channelOptions: Number,
        grayscale : Boolean
    ) {

        var bd:BitmapData = new BitmapData(100, 100, transp, 0x00000000);
        bd.fillRect(new Rectangle(30, 40, 10, 20), 0x80808080);
        bd.perlinNoise(bx, by, numOctaves, seed, stitch, fractalNoise, channelOptions, grayscale);

        var bdmc:MovieClip = mc.createEmptyMovieClip("bitmap", mc.getNextHighestDepth());
        bdmc.attachBitmap(bd, mc.getNextHighestDepth());

        bdmc._x = sx;
        bdmc._y = sy;
    }

    static function plop2(mc : MovieClip, sx : Number, sy : Number,
        bx : Number, by : Number,
        stitch: Boolean,
        offsets
    ) {

        var bd:BitmapData = new BitmapData(100, 100, true, 0x00000000);
        bd.perlinNoise(bx, by, 3, 137, stitch, true, 7, false, offsets);

        var bdmc:MovieClip = mc.createEmptyMovieClip("bitmap", mc.getNextHighestDepth());
        bdmc.attachBitmap(bd, mc.getNextHighestDepth());

        bdmc._x = sx;
        bdmc._y = sy;
    }

    static function main(mc) {

        // different baseX and baseY numbers
        BitmapPerlinNoise.plop(mc,  10, 20, true,  100, 100, 1, 42, false, true, 7);
        BitmapPerlinNoise.plop(mc, 150, 20, true,  500, 500, 1, 42, false, true, 7);
        BitmapPerlinNoise.plop(mc, 310, 20, true,    0,  20, 1, 42, false, true, 7);
        BitmapPerlinNoise.plop(mc, 450, 20, true,   30, 150, 1, 42, false, true, 7);

        // different numOctaves numbers
        BitmapPerlinNoise.plop(mc,  10, 150, true,  100, 100, 0, 3, false, true, 7);
        BitmapPerlinNoise.plop(mc, 150, 150, true,  100, 100, 1, 3, false, true, 7);
        BitmapPerlinNoise.plop(mc, 310, 150, true,  100, 100, 2, 3, false, true, 7);
        BitmapPerlinNoise.plop(mc, 450, 150, true,  100, 100, 3, 3, false, true, 7);

        // different stitch and fractalSum params
        BitmapPerlinNoise.plop(mc,  10, 320, true,  310, 70, 2, 20, false, false, 7);
        BitmapPerlinNoise.plop(mc, 150, 320, true,  130, 70, 2, 20, false, true,  7);
        BitmapPerlinNoise.plop(mc, 310, 320, true,  130, 70, 2, 20, true,  false, 7);
        BitmapPerlinNoise.plop(mc, 450, 320, true,  130, 70, 2, 20, true,  true,  7);

        // different channelOptions
        BitmapPerlinNoise.plop(mc,  10, 450, true, 100, 100, 2, 242, false, true, 1 | 2 | 4, false);
        BitmapPerlinNoise.plop(mc, 150, 450, true, 100, 100, 2, 242, false, true, 2, false);
        BitmapPerlinNoise.plop(mc, 310, 450, true, 100, 100, 2, 242, false, true, 0, false);
        BitmapPerlinNoise.plop(mc, 450, 450, true, 100, 100, 2, 242, false, true, 1 | 2 | 4 | 8, false);
        // more channelOptions
        BitmapPerlinNoise.plop(mc,  10, 620, true, 100, 100, 2, 578, false, true, 1 | 2 | 8, false);
        BitmapPerlinNoise.plop(mc, 150, 620, true, 100, 100, 2, 578, false, true, 1 | 4, false);
        BitmapPerlinNoise.plop(mc, 310, 620, true, 100, 100, 2, 578, false, true, 15, false);
        BitmapPerlinNoise.plop(mc, 450, 620, true, 100, 100, 2, 578, false, true, 8, false);


        // different channelOptions, but greyscale
        BitmapPerlinNoise.plop(mc,  630, 20, true, 100, 100, 2, 242, false, true, 1 | 2 | 4, true);
        BitmapPerlinNoise.plop(mc,  770, 20, true, 100, 100, 2, 242, false, true, 2, true);
        BitmapPerlinNoise.plop(mc,  930, 20, true, 100, 100, 2, 242, false, true, 0, true);
        BitmapPerlinNoise.plop(mc, 1070, 20, true, 100, 100, 2, 242, false, true, 1 | 2 | 4 | 8, true);
        // more channelOptions, but greyscale
        BitmapPerlinNoise.plop(mc,  630, 150, true, 100, 100, 2, 578, false, true, 1 | 2 | 8, true);
        BitmapPerlinNoise.plop(mc,  770, 150, true, 100, 100, 2, 578, false, true, 1 | 4, true);
        BitmapPerlinNoise.plop(mc,  930, 150, true, 100, 100, 2, 578, false, true, 15, true);
        BitmapPerlinNoise.plop(mc, 1070, 150, true, 100, 100, 2, 578, false, true, 8, true);


        // octave offsets
        BitmapPerlinNoise.plop2(mc,  630, 320, 100, 100, false, [new Point(10, 20), new Point(-30, -40), new Point(135, 15)]);
        BitmapPerlinNoise.plop2(mc,  770, 320,  60, 140, false, [new Point(10, 20)]);
        BitmapPerlinNoise.plop2(mc,  930, 320, 100, 100, true,  [new Point(10, 20), new Point(30, 70), new Point(135, 15)]);
        BitmapPerlinNoise.plop2(mc, 1070, 320, 180,  58, true,  [new Point(10, 20), new Point(600, 137)]);

    }
}