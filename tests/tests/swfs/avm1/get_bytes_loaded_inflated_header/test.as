class test {
    static function main() {
        trace("getBytesLoaded(): " + _root.getBytesLoaded());
        trace("getBytesTotal(): " + _root.getBytesTotal());

        var clip = _root.createEmptyMovieClip("clip", 1);
        trace("clip.getBytesLoaded(): " + clip.getBytesLoaded());
        trace("clip.getBytesTotal(): " + clip.getBytesTotal());
    }
}
