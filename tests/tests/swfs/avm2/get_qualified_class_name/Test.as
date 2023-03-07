package {
	public class Test {
	}
}

function normalImport() {
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

	trace(getQualifiedClassName(null));
	trace(getQualifiedClassName(undefined));
}

function avmplusImport() {
	import avmplus.getQualifiedClassName;
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

	trace(getQualifiedClassName(null));
	trace(getQualifiedClassName(undefined));	
}

normalImport();
avmplusImport();