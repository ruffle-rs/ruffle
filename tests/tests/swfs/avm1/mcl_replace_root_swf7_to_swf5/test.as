function replaceMovie(url, includeStart) {
    var d = "closure var";

    var loader = new MovieClipLoader();

    if (includeStart !== false) {
        loader.onLoadStart = function(mc, rest) {
            trace("onLoadStart");
            trace("  d=" + d);
            trace("  rest=" + rest);
            trace("  _target=" + _target);
            trace("  _name=" + _name);
            trace("  _url=" + _url);
            trace("  mc=" + mc);
            trace("  mc._target=" + mc._target);
            trace("  mc._name=" + mc._name);
            trace("  mc._url=" + mc._url);
        };
    }

    loader.onLoadProgress = function(mc, loaded, total, rest) {
        trace("onLoadProgress");
        trace("  d=" + d);
        trace("  loaded=" + loaded);
        trace("  total=" + total);
        trace("  rest=" + rest);
        trace("  _target=" + _target);
        trace("  _name=" + _name);
        trace("  _url=" + _url);
        trace("  mc=" + mc);
        trace("  mc._target=" + mc._target);
        trace("  mc._name=" + mc._name);
        trace("  mc._url=" + mc._url);
    };

    loader.onLoadComplete = function(mc, status, rest) {
        trace("onLoadComplete");
        trace("  d=" + d);
        trace("  status=" + status);
        trace("  rest=" + rest);
        trace("  _target=" + _target);
        trace("  _name=" + _name);
        trace("  _url=" + _url);
        trace("  mc=" + mc);
        trace("  mc._target=" + mc._target);
        trace("  mc._name=" + mc._name);
        trace("  mc._url=" + mc._url);
    };

    loader.onLoadError = function(mc) {
        trace("onLoadError");
        trace("  d=" + d);
        trace("  _target=" + _target);
        trace("  _name=" + _name);
        trace("  _url=" + _url);
        trace("  mc=" + mc);
        trace("  mc._target=" + mc._target);
        trace("  mc._name=" + mc._name);
        trace("  mc._url=" + mc._url);
    };

    loader.onLoadInit = function(mc, rest) {
        trace("onLoadInit");
        trace("  d=" + d);
        trace("  rest=" + rest);
        trace("  _target=" + _target);
        trace("  _name=" + _name);
        trace("  _url=" + _url);
        trace("  mc=" + mc);
        trace("  mc._target=" + mc._target);
        trace("  mc._name=" + mc._name);
        trace("  mc._url=" + mc._url);
    };

    loader.loadClip(url, "_root");
}

_name = "root movie";

trace("Loading child movie");
replaceMovie("child.swf");
