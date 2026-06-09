class Test {
    static function main() {
        // `connect()` with no host, `null`, or `undefined` should fall back to
        // the movie URL's domain (file:// movies use "localhost"). Explicit
        // string hosts must be passed through unchanged.

        trace("// connect() no args");
        var s1 = new XMLSocket();
        s1.connect();

        trace("// connect(null)");
        var s2 = new XMLSocket();
        s2.connect(null);

        trace("// connect(undefined)");
        var s3 = new XMLSocket();
        s3.connect(undefined);

        trace("// connect(\"example.com\", 1234)");
        var s4 = new XMLSocket();
        s4.connect("example.com", 1234);
    }
}
