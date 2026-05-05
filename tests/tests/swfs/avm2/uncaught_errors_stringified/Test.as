package {
    import flash.display.MovieClip;
    import flash.utils.setTimeout;

    public class Test extends MovieClip {
        public function Test() {
            setTimeout(function() {
                setTimeout(function() {
                    setTimeout(function() {
                        var err:Object = {
                            "toString": function() {
                                trace("3 - throwing an error");
                                throw "an error";
                            }
                        };

                        throw err;
                    }, 0);

                    Error.prototype.toString = function() {
                        trace("2 - prototype toString called");
                    };

                    var a = null;
                    // Trigger exception
                    trace(a.field);
                }, 0);

                var err:Object = {
                    "toString": function() {
                        trace("1 - err.toString called");
                    }
                };

                throw err;
            }, 0);
        }
    }
}
