package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
        public function Test()
        {
            testValue("zero", 0);
            testValue("negative zero", -0.0);

            testValue("min positive subnormal", 4.9406564584124654e-324, false);
            testValue("negative min subnormal", -4.9406564584124654e-324, false);

            testValue("NaN", Number.NaN);
            testValue("positive infinity", Number.POSITIVE_INFINITY);
            testValue("negative infinity", Number.NEGATIVE_INFINITY);

            testValue("large positive in-range", 8300000000000000);
            testValue("positive flash limit", 8640000000000000);

            testValue("large negative in-range", -8500000000000000);
            testValue("negative flash limit", -8640000000000000);

            testValue("positive limit + 1", 8640000000000001);
            testValue("negative limit - 1", -8640000000000001);

            testValue("Number.MAX_VALUE", Number.MAX_VALUE);
            testValue("-Number.MAX_VALUE", -Number.MAX_VALUE);

            testValue("WarLight value", 3155378976000000000);
        }

        private function testValue(name:String, value:Number, traceInput:Boolean = true):void
        {
            var date:Date = new Date();

            trace(name);

            if (traceInput)
            {
                trace("input=" + value);
            }

            trace("setTime=" + date.setTime(value));
            trace("getTime=" + date.getTime());
        }
    }
}
