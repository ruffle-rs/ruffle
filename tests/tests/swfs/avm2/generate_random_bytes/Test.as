package {
	public class Test {}
}
import flash.utils.ByteArray;
import flash.crypto.generateRandomBytes;

var ba: ByteArray = generateRandomBytes(5);
trace("// var ba: ByteArray = generateRandomBytes(5);");
trace("// ByteArray length: ");
trace(ba.length);