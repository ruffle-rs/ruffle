package utils {
    import flash.utils.*;
    import parentns.*;
    import childns.*;
    import pending.dormantValue;

    public class Lookup {
        public static function freshChoice():Class {
            return FreshChoice;
        }

        public static function guardedRead(take:Boolean):String {
            if (take) {
                return dormantValue();
            }
            return "skipped";
        }

        public static function choice():Class {
            return Choice;
        }

        public static function makeTimer():Object {
            return new Timer(25, 1);
        }
    }
}
