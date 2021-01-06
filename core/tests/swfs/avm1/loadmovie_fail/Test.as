// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf 
class Test {
    static function main(current) {
        // Regression test for issue #2123
        var mc = current.createEmptyMovieClip("mc", 1);
        var loader = new MovieClipLoader();

        loader.onLoadInit = function(mc2) {
            trace("onLoadInit");
        };

        loader.onLoadError = function(mc2) {
            trace("onLoadError");
        }

        trace("loading...");
        loader.loadClip("bogus.swf", mc);
    }
}