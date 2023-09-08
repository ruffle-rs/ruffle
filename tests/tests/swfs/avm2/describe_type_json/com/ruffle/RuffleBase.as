package com.ruffle {
	import flash.utils.ByteArray;

	public class RuffleBase {
		public function RuffleBase() {
		}

		public var baseMyVar;
		public const baseMyConst;

		[BaseMyCustomMeta]
		public function get baseGetterOnly():String {
			return "";
		};
		public function set baseSetterOnly(val:Boolean) {
		}

		public function get baseGetterAndSetter():Boolean {
			return true;
		}
		public function set baseGetterandSetter(val:Boolean) {
		}

		public function baseMyMethod() {
		}
	}
}