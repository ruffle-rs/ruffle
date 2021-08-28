package {
	public class Test {
	}
}
import flash.utils.getQualifiedClassName;
import com.very.long.namespace.example;
import flash.utils.ByteArray;

trace(getQualifiedClassName(Test));

trace(getQualifiedClassName(flash.utils.ByteArray));

trace(getQualifiedClassName(example));

trace(getQualifiedClassName(new example()));

trace(getQualifiedClassName(int));

trace(getQualifiedClassName(String));

trace(getQualifiedClassName(new flash.utils.ByteArray()));

trace(getQualifiedClassName(new String()));