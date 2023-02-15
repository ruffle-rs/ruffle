package flash.utils {

	import __ruffle__.stub_constructor;

	[Ruffle(InstanceAllocator)]
    public dynamic class Dictionary {
		public function Dictionary(weakKeys:Boolean = false)
		{
			if (weakKeys) {
				stub_constructor("flash.utils.Dictionary", "with weak keys");
			}
		}
    }
}
