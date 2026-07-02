package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            // A 13-digit millisecond timestamp. Before the fix, JSON.parse ran every
            // whole number through f64_to_wrapping_i32, reducing it mod 2^32 and
            // corrupting this to a near-zero value.
            var big = JSON.parse('{"t": 1782219299000}');
            trace(big.t);
            trace(big.t === 1782219299000);
            trace(big.t is int);
            trace(big.t is Number);

            // i32 boundary: the maximum i32 still narrows to int; one past it stays Number.
            var edge = JSON.parse('{"max": 2147483647, "over": 2147483648}');
            trace(edge.max, edge.max is int);
            trace(edge.over, edge.over is int);

            // A value below i32::MIN likewise stays a Number.
            var neg = JSON.parse('{"n": -5000000000}');
            trace(neg.n, neg.n is int);

            // Fractional and small whole numbers are unaffected.
            trace(JSON.parse('{"x": 1.5}').x);
            trace(JSON.parse('{"y": 42}').y is int);
        }
    }
}
