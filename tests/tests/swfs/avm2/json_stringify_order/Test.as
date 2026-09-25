package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            // Test that JSON stringification first lists slots (and const slots),
            // then getters, and finally dynamic properties.
            var struct:Struct = new Struct();
            struct.d1 = 0;
            struct.d3 = 0;
            struct.d2 = 0;
            trace(JSON.stringify(struct).replace(/\d/g, "X"));
        }
    }
}

dynamic class Struct {
    public var s1:int;
    public var s2:int;
    public var s3:int;
    public const s4:int;
    public const s5:int;
    public const s6:int;
    
    public function get m1():int {
        return 0;
    }
    public function get m2():int {
        return 0;
    }
    public function get m3():int {
        return 0;
    }
}
