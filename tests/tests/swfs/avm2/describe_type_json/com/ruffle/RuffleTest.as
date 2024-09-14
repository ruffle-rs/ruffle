package com.ruffle {
	import flash.utils.ByteArray;

	public class RuffleTest extends RuffleBase implements MyInterface {
		public function RuffleTest(first:String, second:Boolean) {
		}

		public var myVar;
		public const myConst;

		[MyCustomMeta]
		public function get getterOnly():String {
			return "";
		};
		public function set setterOnly(val:Boolean) {
		}

		public function get getterAndSetter():Boolean {
			return true;
		}
		public function set getterandSetter(val:Boolean) {
		}

		public function myMethod(first:String, second:ByteArray) {
		}

		public function interfaceMethod() {
		}

	    public function get interfaceGetter() {
	        return null;
	    }
	}
}