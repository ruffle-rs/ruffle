package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            trace("=== Timestamp clipping ===");

            testTime(5.9);
            testTime(-5.9);
            testTime(0.9);
            testTime(-0.9);
            testTime(NaN);
            testTime(Infinity);
            testTime(-Infinity);
            testTime(8640000000000000);
            testTime(8640000000000000 + 1);

            trace("=== setTime ===");

            testSetTime(5.9);
            testSetTime(-5.9);
            testSetTime(0.9);
            testSetTime(-0.9);
            testSetTime(NaN);
            testSetTime(Infinity);
            testSetTime(-Infinity);

            trace("=== Date.UTC fractional fields ===");

            testUTC(2000.9, 0, 1, 0, 0, 0, 0);
            testUTC(2000, 1.9, 1, 0, 0, 0, 0);
            testUTC(2000, -1.9, 1, 0, 0, 0, 0);
            testUTC(2000, 0, 1.9, 0, 0, 0, 0);
            testUTC(2000, 0, -1.9, 0, 0, 0, 0);
            testUTC(2000, 0, 1, 1.9, 2.9, 3.9, 4.9);
            testUTC(2000, 0, 1, -1.9, -2.9, -3.9, -4.9);

            trace("=== Date.UTC invalid fields ===");

            testUTC(NaN, 0, 1, 0, 0, 0, 0);
            testUTC(2000, NaN, 1, 0, 0, 0, 0);
            testUTC(2000, 0, NaN, 0, 0, 0, 0);
            testUTC(Infinity, 0, 1, 0, 0, 0, 0);
            testUTC(2000, -Infinity, 1, 0, 0, 0, 0);

            trace("=== Invalid Date setters ===");

            testInvalidSetter("setUTCFullYear", [2000]);
            testInvalidSetter("setUTCFullYear", [-271821, 3, 20]);
            testInvalidSetter("setUTCMonth", [0]);
            testInvalidSetter("setUTCDate", [1]);
            testInvalidSetter("setUTCHours", [0]);
            testInvalidSetter("setUTCMinutes", [0]);
            testInvalidSetter("setUTCSeconds", [0]);
            testInvalidSetter("setUTCMilliseconds", [0]);

            trace("=== Invalid Date setFullYear ===");

            var localDate:Date = new Date(NaN);
            trace("result: " + localDate.setFullYear(2000, 0, 1));
            trace("time: " + localDate.getTime());
            trace("fullYear: " + localDate.getFullYear());
            trace("month: " + localDate.getMonth());
            trace("date: " + localDate.getDate());
        }

        private function testTime(value:Number):void {
            var date:Date = new Date(value);
            trace("new Date(" + value + "): " + date.getTime());
        }

        private function testSetTime(value:Number):void {
            var date:Date = new Date(0);
            trace("setTime(" + value + "): " + date.setTime(value));
            trace("stored: " + date.getTime());
        }

        private function testUTC(
            year:Number,
            month:Number,
            day:Number,
            hour:Number,
            minute:Number,
            second:Number,
            millisecond:Number
        ):void {
            var value:Number = Date.UTC(
                year, month, day,
                hour, minute, second, millisecond
            );

            trace(
                "UTC(" +
                year + "," + month + "," + day + "," +
                hour + "," + minute + "," + second + "," +
                millisecond + "): " + value
            );

            var date:Date = new Date(value);
            trace(
                "fields: " +
                date.getUTCFullYear() + "," +
                date.getUTCMonth() + "," +
                date.getUTCDate() + "," +
                date.getUTCHours() + "," +
                date.getUTCMinutes() + "," +
                date.getUTCSeconds() + "," +
                date.getUTCMilliseconds()
            );
        }

        private function testInvalidSetter(name:String, args:Array):void {
            var date:Date = new Date(NaN);
            var result:*;

            if (args.length == 1) {
                result = date[name](args[0]);
            } else if (args.length == 3) {
                result = date[name](args[0], args[1], args[2]);
            }

            trace(name + "(" + args.join(",") + "): " + result);
            trace("stored: " + date.getTime());
        }
    }
}
