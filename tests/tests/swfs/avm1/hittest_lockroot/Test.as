// Compile with: mtasc -main -header 550:400:30 Test.as -swf test.swf

class Test {
    static function main(root) {
        initClip(root);
        testChildHits(root.child);
    }

    static function initClip(clip) {
        clip.createEmptyMovieClip("child", clip.getNextHighestDepth());
        initChild(clip.child, 275, 200);
    }

    static function initChild(cclip, cx, cy) {
        cclip.createEmptyMovieClip("hit", cclip.getNextHighestDepth());
        cclip._x = cx;
        cclip._y = cy;
        initHit(cclip.hit, 100, 100);
    }

    static function initHit(hclip, hwidth, hheight) {
        hclip.beginFill(0xFF0000);
        hclip.moveTo(hwidth * -0.5, hheight * -0.5);
        hclip.lineTo(hwidth *  0.5, hheight * -0.5);
        hclip.lineTo(hwidth *  0.5, hheight *  0.5);
        hclip.lineTo(hwidth * -0.5, hheight *  0.5);
        hclip.endFill();
    }

    static function testChildHits(cclip) {
        trace("child hit: " + cclip); // Make sure everything exists
        var point = {
            x: 0,
            y: 0
        };
        testPointHits(cclip, point); // Should all fail
        cclip.hit.localToGlobal(point);
        testPointHits(cclip, point); // Should all succeed
    }

    static function testPointHits(mc, point) {
        trace("point.x: " + point.x + "; point.y: " + point.y);
        testRootLockHits(mc, point, false);
        testRootLockHits(mc, point, true);
    }

    static function testRootLockHits(mc, point, lockRoot) {
        mc._lockroot = lockRoot;
        trace("_lockroot: " + mc._lockroot);
        testHits(mc.hit, point);
    }

    static function testHits(mc, point) {
        testHit(mc, point, false);
        testHit(mc, point, true);
    }

    static function testHit(mc, point, shapeFlag) {
        trace("hitTest (shapeFlag = " + shapeFlag + "): " + mc.hitTest(point.x, point.y, shapeFlag));
    }
}