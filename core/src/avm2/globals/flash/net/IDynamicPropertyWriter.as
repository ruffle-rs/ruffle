package flash.net {
    public interface IDynamicPropertyWriter {
        function writeDynamicProperties(obj: Object, output: IDynamicPropertyOutput): void;
    }
}
