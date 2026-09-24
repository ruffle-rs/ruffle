package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            trace("=== multi-argument constructor around upper bound ===");

            testDate(275760, 8, 12, 23, 59, 59, 999);
            testDate(275760, 8, 13, 0, 0, 0, 0);
            testDate(275760, 8, 13, 0, 0, 0, 1);
            testDate(275760, 8, 13, 1, 0, 0, 0);
            testDate(275760, 8, 14, 0, 0, 0, 0);
            testDate(275761, 0, 1, 0, 0, 0, 0);
            testDate(300000, 0, 1, 0, 0, 0, 0);
            testDate(1000000, 0, 1, 0, 0, 0, 0);

            trace("=== operations on out-of-range constructed date ===");

            var d:Date = new Date(275760, 8, 13, 1, 0, 0, 0);
            dump("initial", d);

            trace("valueOf: " + d.valueOf());
            trace("toString: " + d.toString());
            trace("toUTCString: " + d.toUTCString());

            trace("setUTCMilliseconds(0): " + d.setUTCMilliseconds(0));
            dump("after setUTCMilliseconds", d);

            d = new Date(275760, 8, 13, 1, 0, 0, 0);
            trace("setTime(getTime()): " + d.setTime(d.getTime()));
            dump("after setTime", d);

            trace("=== timestamp constructor with same values ===");

            testTimestamp(8640000000000000);
            testTimestamp(8640000000000001);
            testTimestamp(8640000003600000);
            testTimestamp(8640000057600000);
        }

        private function testDate(
            year:Number,
            month:Number,
            day:Number,
            hour:Number,
            minute:Number,
            second:Number,
            millisecond:Number
        ):void {
            var d:Date = new Date(
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond
            );

            trace(
                "new Date(" +
                year + "," +
                month + "," +
                day + "," +
                hour + "," +
                minute + "," +
                second + "," +
                millisecond + ")"
            );

            dump("result", d);
        }

        private function testTimestamp(value:Number):void {
            var d:Date = new Date(value);
            trace("new Date(timestamp " + value + ")");
            dump("result", d);
        }

        private function dump(label:String, d:Date):void {
            trace(label);
            trace("time: " + d.getTime());
            trace("timezoneOffset: " + d.getTimezoneOffset());

            trace(
                "local: " +
                d.getFullYear() + "," +
                d.getMonth() + "," +
                d.getDate() + "," +
                d.getHours() + "," +
                d.getMinutes() + "," +
                d.getSeconds() + "," +
                d.getMilliseconds()
            );

            trace(
                "utc: " +
                d.getUTCFullYear() + "," +
                d.getUTCMonth() + "," +
                d.getUTCDate() + "," +
                d.getUTCHours() + "," +
                d.getUTCMinutes() + "," +
                d.getUTCSeconds() + "," +
                d.getUTCMilliseconds()
            );

            trace("---");
        }
    }
}
