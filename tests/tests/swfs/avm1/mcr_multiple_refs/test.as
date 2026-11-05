class TestFinal {
    static function main() {
        var root = _root;

        // Test multiple MovieClipReferences pointing to the same MovieClip
        // after reference laundering (passing MCR through a function: MCR -> Object -> MCR).
        // This verifies that when two MCRs with invalidated caches both resolve
        // to the same clip, they don't interfere with each other.

        // Test 1: Two refs via same path, both laundered after remove+recreate
        trace("=== Test 1: Two refs same path ===");
        root.createEmptyMovieClip("clip", 1);
        var ref1 = root.clip;
        var ref2 = root.clip;
        root.clip.removeMovieClip();
        root.createEmptyMovieClip("clip", 1);
        ref1 = launder(ref1);
        ref2 = launder(ref2);
        ref1._x = 100;
        trace(root.clip._x);
        trace(ref2._x);
        ref2._x = 200;
        trace(root.clip._x);
        trace(ref1._x);
        root.clip.removeMovieClip();

        // Test 2: Sequential launder - ref1 laundered, then ref2 laundered,
        // then ref1 laundered again
        trace("=== Test 2: Re-launder after overwrite ===");
        root.createEmptyMovieClip("clip", 1);
        ref1 = root.clip;
        root.clip.removeMovieClip();
        root.createEmptyMovieClip("clip", 1);
        ref1 = launder(ref1);
        ref1._x = 10;
        trace(root.clip._x);
        ref2 = root.clip;
        root.clip.removeMovieClip();
        root.createEmptyMovieClip("clip", 1);
        ref2 = launder(ref2);
        ref2._x = 20;
        trace(root.clip._x);
        ref1 = launder(ref1);
        ref1._x = 30;
        trace(root.clip._x);
        trace(ref2._x);
        root.clip.removeMovieClip();

        // Test 3: toString and typeof after multiple launders
        trace("=== Test 3: toString and typeof ===");
        root.createEmptyMovieClip("mc", 3);
        var a = root.mc;
        root.mc.removeMovieClip();
        root.createEmptyMovieClip("mc", 3);
        a = launder(a);
        trace(a);
        var b = root.mc;
        root.mc.removeMovieClip();
        root.createEmptyMovieClip("mc", 3);
        b = launder(b);
        trace(b);
        a = launder(a);
        trace(a);
        trace(typeof(a));
        trace(typeof(b));
        trace(a == b);
        root.mc.removeMovieClip();
    }

    static function launder(x) {
        return x;
    }
}
