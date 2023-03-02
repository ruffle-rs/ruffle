package flash.utils {
    public interface IExternalizable {
        function readExternal(input:IDataInput):void;

        function writeExternal(output:IDataOutput):void;
    }
}
