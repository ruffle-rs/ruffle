package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            var vec:Vector.<int> = new Vector.<int>();

            trace("NUMERICAL INDEXING");

            try {
                trace("Attempting to access vec[8000000000] (8 x 10^9");
                vec[8000000000];
            } catch(e:Error) {
                trace("Got " + e);
            }

            try {
                trace("Attempting to access vec[4 294 967 296] (2^32)");
                vec[4294967296];
            } catch(e:Error) {
                trace("Got " + e);
            }

            try {
                trace("Attempting to access vec[4294967295] (2^32 - 1)");
                vec[4294967295];
            } catch(e:Error) {
                trace("Got " + e);
            }

            try {
                trace("Attempting to access vec[4294967294] (2^32 - 2)");
                vec[4294967294];
            } catch(e:Error) {
                trace("Got " + e);
            }

            trace("STRING INDEXING");

            try {
                trace("Attempting to access vec['8000000000']");
                vec['8000000000'];
            } catch(e:Error) {
                trace("Got " + e);
            }

            try {
                trace("Attempting to access vec['4294967296']");
                vec['4294967296'];
            } catch(e:Error) {
                trace("Got " + e);
            }

            try {
                trace("Attempting to access vec['4294967295']");
                vec['4294967295'];
            } catch(e:Error) {
                trace("Got " + e);
            }

            try {
                trace("Attempting to access vec['4294967294']");
                vec['4294967294'];
            } catch(e:Error) {
                trace("Got " + e);
            }
        }
    }
}
