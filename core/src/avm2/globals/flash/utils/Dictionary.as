package flash.utils {

	import __ruffle__.log_warn;

	[Ruffle(InstanceAllocator)]
    public dynamic class Dictionary {
		public function Dictionary(weakKeys:Boolean = false)
		{
			if (weakKeys) {
				log_warn("weak keys for Dictionary are not implemented");
			}
		}
    }
}
