package {

import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        var stringJsons = [
            '{"t": 1782219299000}',

            // around the i32 boundary
            '{"t": 2147483647}',
            '{"t": 2147483648}',
            '{"t": -2147483648}',
            '{"t": -2147483649}',

            // around the u32 boundary
            '{"t": 0}',
            '{"t": -0}',
            '{"t": 4294967295}',
            '{"t": 4294967296}',
            '{"t": -1}',

            // around 2^53, the largest integer exactly representable in a double
            '{"t": 9007199254740991}',
            '{"t": 9007199254740992}',
            '{"t": 9007199254740993}',
            '{"t": -9007199254740991}',
            '{"t": -9007199254740992}',
            '{"t": -9007199254740993}',

            // extremes of double range
            '{"t": 1.7976931348623157e308}',
            '{"t": 1.7976931348623159e308}',
            '{"t": 5e-324}',
            '{"t": 1e-400}',

            // around the i64/u64 boundaries
            '{"t": 9223372036854775807}',
            '{"t": 9223372036854775808}',
            '{"t": 18446744073709551615}',
            '{"t": 18446744073709551616}',
            '{"t": -9223372036854775808}',
            '{"t": -9223372036854775809}',

            // whole numbers written with a decimal point or exponent
            '{"t": 1.0}',
            '{"t": 2147483647.0}',
            '{"t": 100.00}',
            '{"t": 1e21}',
            '{"t": 1e2}',

            // ordinary fractional values
            '{"t": 0.1}',
            '{"t": 1.5}',

            // around the boundary of a float's 24-bit mantissa
            '{"t": 16777216}',
            '{"t": 16777217}',

            // around the boundary of a float's max finite value
            '{"t": 3.4028235e38}',
            '{"t": 3.4028236e38}',

            // around the boundary of a float's smallest normal/subnormal values
            '{"t": 1.1754944e-38}',
            '{"t": 1.401298464324817e-45}',

            // more significant decimal digits than a float's mantissa can hold
            '{"t": 123456789.123456789}',

            // different exponent notations
            '{"t": 1E5}',
            '{"t": 1e+5}',
            '{"t": 1e-5}',
            '{"t": 1e05}',
            '{"t": 1.5E+2}',
            '{"t": 2e0}',
            '{"t": 0e10}',
            '{"t": -1e5}',
            '{"t": -0e0}',

            '{"t": -5000000000}',

            // leading zero / sign forms not allowed by the JSON number grammar
            '{"t": 01}',
            '{"t": 007}',
            '{"t": 010}',
            '{"t": +5}',
            '{"t": +5.5}',
            '{"t": .5}',
            '{"t": -.5}',
            '{"t": 5.}',
            '{"t": -5.}',
            '{"t": 5e}',
            '{"t": 5e+}',
            '{"t": 5e-}',

            // numeric literal syntax from ActionScript/JS that JSON doesn't define
            '{"t": 0x1F}',
            '{"t": 0o17}',
            '{"t": 0b101}',
            '{"t": 1_000_000}',
            '{"t": Infinity}',
            '{"t": -Infinity}',
            '{"t": NaN}',

            // very long digit sequences
            '{"t": 111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111}',
            '{"t": 1.111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111}',

            // a full-width digit
            '{"t": ５}',

            // whitespace inside the number token itself, rather than around it
            '{"t": 1 . 5}',
            '{"t": 1e 5}'
        ]

        for each (var stringJson in stringJsons) {
            try {
                var parsed = JSON.parse(stringJson);
                trace(stringJson + " -> " + parsed.t);
            } catch (e) {
                trace(stringJson + " -> threw " + e.getStackTrace());
            }
        }
    }
}

}
