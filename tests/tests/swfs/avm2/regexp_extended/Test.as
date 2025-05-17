package {
    import flash.display.Sprite;

    public class Test extends Sprite {

        function Test() {
            function run(url) {
                trace("URL: " + url);

                var regexp = /(?#comment) ((?P<protocol>[a-zA-Z]+: \/\/) (?P<host>[^:\/]*) (:(?P<port>\d+))?)? (?P<path>[^?]*)? ((?P<query>.*))? /x;
                var match = regexp.exec(url);
                trace("match: " + match);
                trace('match["protocol"]: ' + match["protocol"]);
                trace('match["host"]: ' + match["host"]);
                trace('match["port"]: ' + match["port"]);
                trace('match["path"]: ' + match["path"]);
                trace('match["query"]: ' + match["query"]);

                trace();
            }

            run("");
            run("http://");
            run("http://example.org");
            run("http://example.org/abc");
            run("http://example.org:80/abc");
            run("http://example.org/abc?hey");
        }

    }
}
