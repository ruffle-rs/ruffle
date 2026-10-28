package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            trace("=== Date.UTC construction ===");

            testUTC("minimum", -271821, 3, 20, 0, 0, 0, 0);
            testUTC("before minimum", -271821, 3, 19, 23, 59, 59, 999);
            testUTC("minimum + 1ms", -271821, 3, 20, 0, 0, 0, 1);

            testUTC("chrono negative boundary", -262143, 0, 1, 0, 0, 0, 0);
            testUTC("year -1", -1, 0, 1, 0, 0, 0, 0);
            testUTC("year 0", 0, 0, 1, 0, 0, 0, 0);
            testUTC("year 99", 99, 0, 1, 0, 0, 0, 0);
            testUTC("year 100", 100, 0, 1, 0, 0, 0, 0);

            testUTC("chrono positive boundary", 262143, 0, 1, 0, 0, 0, 0);

            testUTC("maximum - 1ms", 275760, 8, 12, 23, 59, 59, 999);
            testUTC("maximum", 275760, 8, 13, 0, 0, 0, 0);
            testUTC("after maximum", 275760, 8, 13, 0, 0, 0, 1);

            trace("=== Normalization ===");

            testUTC("month overflow", 275759, 20, 13, 0, 0, 0, 0);
            testUTC("month negative", -271820, -9, 20, 0, 0, 0, 0);
            testUTC("day overflow to maximum", 275760, 7, 44, 0, 0, 0, 0);
            testUTC("hour overflow to maximum", 275760, 8, 12, 24, 0, 0, 0);
            testUTC("millisecond underflow", -271821, 3, 20, 0, 0, 0, -1);

            trace("=== setUTCFullYear ===");

            testSetUTCFullYear("set minimum", -271821, 3, 20);
            testSetUTCFullYear("set before minimum", -271821, 3, 19);
            testSetUTCFullYear("set chrono negative boundary", -262143, 0, 1);
            testSetUTCFullYear("set chrono positive boundary", 262143, 0, 1);
            testSetUTCFullYear("set maximum", 275760, 8, 13);
            testSetUTCFullYear("set after maximum", 275760, 8, 14);

            trace("=== setters from boundary dates ===");

            var minDate:Date = new Date(-8640000000000000);
            trace("minimum setUTCMilliseconds(-1): " + minDate.setUTCMilliseconds(-1));
            dump(minDate);

            var maxDate:Date = new Date(8640000000000000);
            trace("maximum setUTCMilliseconds(1): " + maxDate.setUTCMilliseconds(1));
            dump(maxDate);
        }

        private function testUTC(
            label:String,
            year:Number,
            month:Number,
            day:Number,
            hour:Number,
            minute:Number,
            second:Number,
            millisecond:Number
        ):void {
            trace(label);

            var time:Number = Date.UTC(
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond
            );

            trace("Date.UTC: " + time);

            var date:Date = new Date(time);
            dump(date);
        }

        private function testSetUTCFullYear(
            label:String,
            year:Number,
            month:Number,
            day:Number
        ):void {
            trace(label);

            var date:Date = new Date(0);
            var result:Number = date.setUTCFullYear(year, month, day);

            trace("result: " + result);
            dump(date);
        }

        private function dump(date:Date):void {
            trace("time: " + date.getTime());
            trace("utcFullYear: " + date.getUTCFullYear());
            trace("utcMonth: " + date.getUTCMonth());
            trace("utcDate: " + date.getUTCDate());
            trace("utcDay: " + date.getUTCDay());
            trace("utcHours: " + date.getUTCHours());
            trace("utcMinutes: " + date.getUTCMinutes());
            trace("utcSeconds: " + date.getUTCSeconds());
            trace("utcMilliseconds: " + date.getUTCMilliseconds());
            trace("---");
        }
    }
}
