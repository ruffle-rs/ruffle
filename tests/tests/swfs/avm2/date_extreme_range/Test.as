package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            var values:Array = [
                -8640000000000001,
                -8640000000000000,
                -8639999999999999,
                -8500000000000000,
                -8334601228800000,
                -8334601228799999,
                0,
                8210266876799999,
                8210266876800000,
                8300000000000000,
                8500000000000000,
                8639999999999999,
                8640000000000000,
                8640000000000001
            ];

            for each (var value:Number in values) {
                var date:Date = new Date(value);

                trace("value: " + value);
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
}
