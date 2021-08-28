package {
	public class Test {
	}
}
import flash.utils.getDefinitionByName;


trace(getDefinitionByName("int"));

trace(getDefinitionByName("Test"));

trace(getDefinitionByName("flash.utils.getDefinitionByName"));

trace(getDefinitionByName("flash.utils.ByteArray"));

trace(getDefinitionByName("Object"));

trace(getDefinitionByName("flash.utils.Endian").LITTLE_ENDIAN)

getDefinitionByName("trace")("Hello World");

trace(getDefinitionByName("::Object"));

trace(getDefinitionByName(".String"));
