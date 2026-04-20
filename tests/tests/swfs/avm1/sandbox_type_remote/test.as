trace("Current sandbox type: " + System.security.sandboxType);

var networkMc = _root.createEmptyMovieClip("network_mc", _root.getNextHighestDepth());

var loader = new MovieClipLoader();
loader.onLoadInit = function(target, status) {
    var noNetworkMc = _root.createEmptyMovieClip("no_network_mc", _root.getNextHighestDepth());
    noNetworkMc.loadMovie("http://localhost:8000/test-no-network.swf");
}
loader.loadClip("http://localhost:8000/test-network.swf", networkMc);
