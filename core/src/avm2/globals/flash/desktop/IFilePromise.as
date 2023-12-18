package flash.desktop {
    import flash.utils.IDataInput;
    import flash.events.ErrorEvent;

    [API("668")] // AIR 2.0
    public interface IFilePromise {
        function get isAsync():Boolean;
        function get relativePath():String;
        function close():void;
        function open():IDataInput;
        function reportError(e:ErrorEvent):void

    }
}