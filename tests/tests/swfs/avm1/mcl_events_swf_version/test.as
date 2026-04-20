function testMovie(url, callback, includeStart) {
    var d = _root.getNextHighestDepth();
    var mc = _root.createEmptyMovieClip("mc" + d, d);

    var loader = new MovieClipLoader();

    if (includeStart !== false) {
        loader.onLoadStart = function() {
            trace("onLoadStart");
            trace("  d=" + d);
            trace("  _target=" + _target);
            trace("  _name=" + _name);
            trace("  _url=" + _url);
            trace("  mc=" + mc);
            trace("  mc._target=" + mc._target);
            trace("  mc._name=" + mc._name);
            trace("  mc._url=" + mc._url);
        };
    }

    loader.onLoadProgress = function() {
        trace("onLoadProgress");
        trace("  d=" + d);
        trace("  _target=" + _target);
        trace("  _name=" + _name);
        trace("  _url=" + _url);
        trace("  mc=" + mc);
        trace("  mc._target=" + mc._target);
        trace("  mc._name=" + mc._name);
        trace("  mc._url=" + mc._url);
        loader.onLoadProgress = function() {};
    };

    loader.onLoadComplete = function() {
        trace("onLoadComplete");
        trace("  d=" + d);
        trace("  _target=" + _target);
        trace("  _name=" + _name);
        trace("  _url=" + _url);
        trace("  mc=" + mc);
        trace("  mc._target=" + mc._target);
        trace("  mc._name=" + mc._name);
        trace("  mc._url=" + mc._url);
    };

    loader.onLoadError = function() {
        trace("onLoadError");
        trace("  d=" + d);
        trace("  _target=" + _target);
        trace("  _name=" + _name);
        trace("  _url=" + _url);
        trace("  mc=" + mc);
        trace("  mc._target=" + mc._target);
        trace("  mc._name=" + mc._name);
        trace("  mc._url=" + mc._url);
        callback();
    };

    loader.onLoadInit = function() {
        trace("onLoadInit");
        trace("  d=" + d);
        trace("  _target=" + _target);
        trace("  _name=" + _name);
        trace("  _url=" + _url);
        trace("  mc=" + mc);
        trace("  mc._target=" + mc._target);
        trace("  mc._name=" + mc._name);
        trace("  mc._url=" + mc._url);
        callback();
    };

    loader.loadClip(url, mc);
}

_name = "root movie";

testMovie("child5.swf", function() {
    testMovie("child6.swf", function() {
        testMovie("child7.swf", function() {
            testMovie("child8.swf", function() {
                testMovie("child9.swf", function() {
                    testMovie("child10.swf", function() {
                        testMovie("non_existent.swf", function() {
                            trace("Done");
                        }, false);
                    });
                });
            });
        });
    });
});
