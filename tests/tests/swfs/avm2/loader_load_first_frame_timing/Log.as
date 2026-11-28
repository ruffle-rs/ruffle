package {
public class Log {
    public static var enabled:Boolean = false;
    public static var loadeeName:String;

    public static function t(s:String):void {
        if (enabled) trace(s);
    }
}
}
