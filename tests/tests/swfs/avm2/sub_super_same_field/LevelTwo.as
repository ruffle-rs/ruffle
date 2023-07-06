package {
	public class LevelTwo extends LevelOne {
		// Note - the final 'test.swf' has been manually edited
		// to remove '_SUFFIX', so that these fields have the same
		// names as the fields in LevelOne
		public var pubSameName_SUFFIX2 = "pubSameName-LevelTwo";
		internal var internalSameName_SUFFIX2 = "pubSameName-LevelTwo";
		
		override public function print() {
			super.print();
			trace("In LevelTwo: " + this + " this.pubSameName = " + this.pubSameName + " this.internalSameName = " + this.internalSameName);
		}
	}
}