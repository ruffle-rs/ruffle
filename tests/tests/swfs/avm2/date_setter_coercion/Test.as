package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            var d:Date = new Date(NaN);
            trace("invalid UTC result:", d.setUTCHours({
                valueOf: function():Number {
                    trace("coerced UTC hour");
                    return 1;
                }
            }));

            try {
                d.setHours({
                    valueOf: function():Number {
                        throw new Error("coerced local hour");
                    }
                });
            } catch (e:Error) {
                trace(e.message);
            }

            trace("local NaN stops coercion:", d.setHours(NaN, {
                valueOf: function():Number {
                    trace("unexpected minute coercion");
                    return 1;
                }
            }));

            trace("UTC setter sees the date after both conversions:", d.setUTCHours({
                valueOf: function():Number {
                    trace("coerced first UTC argument");
                    d.setTime(Date.UTC(2000, 1, 3, 4, 5, 6, 7));
                    return 8;
                }
            }, {
                valueOf: function():Number {
                    trace("coerced second UTC argument");
                    d.setTime(Date.UTC(2001, 2, 4, 5, 6, 7, 8));
                    return 9;
                }
            }) == Date.UTC(2001, 2, 4, 8, 9, 7, 8));

            d.setMonth({
                valueOf: function():Number {
                    d.setTime(new Date(2002, 3, 5, 6, 7, 8, 9).getTime());
                    return 6;
                }
            });
            trace("local setter sees changed date:", d.getTime() == new Date(2002, 6, 5, 6, 7, 8, 9).getTime());

            d.setTime(0);
            trace("UTC setter sees invalidation:", d.setUTCDate({
                valueOf: function():Number {
                    d.setTime(NaN);
                    return 1;
                }
            }));
        }
    }
}
