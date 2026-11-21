package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            var posZero:Number = 0.0;
            var negZero:Number = (-1.0) / Infinity;
            var intZero:int = 0;
            var nan:Number = NaN;

            var posZeroMethod1:Function = posZero.toPrecision;
            var posZeroMethod2:Function = posZero.toPrecision;
            var negZeroMethod1:Function = negZero.toPrecision;
            var negZeroMethod2:Function = negZero.toPrecision;
            var intZeroMethod1:Function = intZero.toPrecision;
            var intZeroMethod2:Function = intZero.toPrecision;
            var nanMethod1:Function = nan.toPrecision;
            var nanMethod2:Function = nan.toPrecision;

            trace("(0.0).toPrecision === (0.0).toPrecision: " + (posZeroMethod1 === posZeroMethod2));
            trace("(-0.0).toPrecision === (-0.0).toPrecision: " + (negZeroMethod1 === negZeroMethod2));
            trace("(0).toPrecision === (0).toPrecision: " + (intZeroMethod1 === intZeroMethod2));
            trace("(NaN).toPrecision === (NaN).toPrecision: " + (nanMethod1 === nanMethod2));

            trace("");

            trace("(0.0).toPrecision === (-0.0).toPrecision: " + (posZeroMethod1 === negZeroMethod1));
            trace("(0.0).toPrecision === (0).toPrecision: " + (posZeroMethod1 === intZeroMethod1));
            trace("(-0.0).toPrecision === (0).toPrecision: " + (negZeroMethod1 === intZeroMethod1));

            trace("");

            trace("(0.0).toPrecision === (NaN).toPrecision: " + (posZeroMethod1 === nanMethod1));
            trace("(-0.0).toPrecision === (NaN).toPrecision: " + (negZeroMethod1 === nanMethod1));
            trace("(0).toPrecision === (NaN).toPrecision: " + (intZeroMethod1 === nanMethod1));
        }
    }
}
