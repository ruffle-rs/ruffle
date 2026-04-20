package flash.display3D {
    public final class Context3DClearMask {
        public static const ALL:int = COLOR | DEPTH | STENCIL;
        public static const COLOR:int =   1 << 0;
        public static const DEPTH:int =   1 << 1;
        public static const STENCIL:int = 1 << 2;
    }
}
