package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            var values:Array = [
                8639999999998000,
                8639999999999000,
                8639999999999500,
                8639999999999900,
                8639999999999950,
                8639999999999990,
                8639999999999991,
                8639999999999992,
                8639999999999993,
                8639999999999994,
                8639999999999995,
                8639999999999996,
                8639999999999997,
                8639999999999998,
                8639999999999999,
                8640000000000000
            ];

            for each (var value:Number in values) {
                var d:Date = new Date(value);

                trace("value: " + value);
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
}
