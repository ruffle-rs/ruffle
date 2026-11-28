package {
public class Log {
    public static var frame:int = 0;
    public static var loadeeNames:Array = [];

    public static function t(s:String):void {
        trace("frame " + frame + ": " + s);
    }
}
}
