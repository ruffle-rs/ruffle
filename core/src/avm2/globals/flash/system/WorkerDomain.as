package flash.system {
    public final class WorkerDomain {
        public static const isSupported: Boolean = false;

        public function WorkerDomain() {
            throw new ArgumentError("Error #2012: WorkerDomain$ class cannot be instantiated.", 2012)
        }
    }
}