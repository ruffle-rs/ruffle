package {
import flash.display.*;
import flash.text.*;
import flash.filters.*;
import flash.geom.*;

public class Test extends Sprite {
    public function Test() {
        trace("Input");
        printBitmapData(createBitmapData());

        testColorMatrixFilter();
        testGradientBevelFilter();
        testGradientGlowFilter();

        trace("Done");
    }

    private function testColorMatrixFilter():void {
        trace("ColorMatrixFilter");

        var dest:BitmapData = new BitmapData(4,4);

        var array:Array = new Array(20);
        array[0] = 1;
        array[18] = 1;

        dest.applyFilter(createBitmapData(), new Rectangle(0, 0, 4, 4), new Point(0, 0), new ColorMatrixFilter(array));

        printBitmapData(dest);
    }

    private function testGradientBevelFilter():void {
        trace("GradientBevelFilter");

        var dest:BitmapData = new BitmapData(4,4);

        var colors:Array = new Array(4);
        colors[0] = 0xff0000;
        colors[2] = 0x00ff00;
        colors[3] = 0x0000ff;

        var alphas:Array = new Array(4);
        alphas[0] = 1;
        alphas[1] = 1;
        alphas[2] = 1;

        var ratios:Array = new Array(4);
        ratios[0] = 0;
        ratios[1] = 1;
        ratios[3] = 3;

        dest.applyFilter(createBitmapData(), new Rectangle(0, 0, 4, 4), new Point(0, 0), new GradientBevelFilter(0, 0, colors, alphas, ratios, 0, 0));

        printBitmapData(dest);
    }

    private function testGradientGlowFilter():void {
        trace("GradientGlowFilter");

        var dest:BitmapData = new BitmapData(4,4);

        var colors:Array = new Array(4);
        colors[0] = 0xff0000;
        colors[2] = 0x00ff00;
        colors[3] = 0x0000ff;

        var alphas:Array = new Array(4);
        alphas[0] = 1;
        alphas[1] = 1;
        alphas[2] = 1;

        var ratios:Array = new Array(4);
        ratios[0] = 0;
        ratios[1] = 1;
        ratios[3] = 3;

        dest.applyFilter(createBitmapData(), new Rectangle(0, 0, 4, 4), new Point(0, 0), new GradientGlowFilter(0, 0, colors, alphas, ratios, 0, 0));

        printBitmapData(dest);
    }

    private function createBitmapData():BitmapData {
        var bd:BitmapData = new BitmapData(4, 4);

        bd.setPixel(0, 0, 0x000000);
        bd.setPixel(0, 1, 0xff0000);
        bd.setPixel(1, 0, 0x00ff00);
        bd.setPixel(1, 1, 0x0000ff);
        bd.setPixel(2, 0, 0x00ffff);
        bd.setPixel(2, 1, 0xff00ff);
        bd.setPixel(3, 0, 0xffff00);
        bd.setPixel(3, 1, 0xffffff);

        bd.setPixel32(0, 2, 0x80000000);
        bd.setPixel32(0, 3, 0x80ff0000);
        bd.setPixel32(1, 2, 0x8000ff00);
        bd.setPixel32(1, 3, 0x800000ff);
        bd.setPixel32(2, 2, 0x8000ffff);
        bd.setPixel32(2, 3, 0x80ff00ff);
        bd.setPixel32(3, 2, 0x80ffff00);
        bd.setPixel32(3, 3, 0x80ffffff);

        return bd;
    }

    private function printBitmapData(bd:BitmapData):void {
        trace(" BitmapData " + bd.width + "x" + bd.height + ":");
        for (var j:int = 0; j < bd.height; ++j) {
            var row:String = " ";
            for (var i:int = 0; i < bd.width; ++i) {
                row += " " + bd.getPixel32(i, j).toString(16);
            }
            trace(row);
        }
    }
}
}
