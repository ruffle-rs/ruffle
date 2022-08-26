package {
    public final class Math {
        public static const E: Number = 2.718281828459045;
        public static const LN10: Number = 2.302585092994046;
        public static const LN2: Number = 0.6931471805599453;
        public static const LOG10E: Number = 0.4342944819032518;
        public static const LOG2E: Number = 1.442695040888963387;
        public static const PI: Number = 3.141592653589793;
        public static const SQRT1_2: Number = 0.7071067811865476;
        public static const SQRT2: Number = 1.4142135623730951;

        public static native function abs(x: Number): Number;
        public static native function acos(x: Number): Number;
        public static native function asin(x: Number): Number;
        public static native function atan(x: Number): Number;
        public static native function ceil(x: Number): Number;
        public static native function cos(x: Number): Number;
        public static native function exp(x: Number): Number;
        public static native function floor(x: Number): Number;
        public static native function log(x: Number): Number;
        public static native function round(x: Number): Number;
        public static native function sin(x: Number): Number;
        public static native function sqrt(x: Number): Number;
        public static native function tan(x: Number): Number;
        public static native function atan2(y: Number, x: Number): Number;
        public static native function pow(x: Number, y: Number): Number;

        // This is a hacky way to specify `-Infinity` as a default value.
        private static const NegInfinity: Number = -1 / 0;
        public static native function max(x: Number = NegInfinity, y: Number = NegInfinity, ...rest): Number;
        public static native function min(x: Number = Infinity, y: Number = Infinity, ...rest): Number;

        public static native function random(): Number;
    }
}
