package flash.utils {

	import __ruffle__.stub_constructor;

	[Ruffle(InstanceAllocator)]
    public dynamic class Dictionary {
		public function Dictionary(weakKeys:Boolean = false)
		{
			if (weakKeys) {
				this.make_weak();
			}
		}

		private native function make_weak():void;
    }
}
