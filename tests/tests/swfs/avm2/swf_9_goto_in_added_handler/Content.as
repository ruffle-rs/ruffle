package {
    import flash.display.MovieClip;

    public dynamic class Content extends MovieClip {
        public var comp:Comp;

        public function Content() {
            addFrameScript(2, frame3);
        }

        public function frame3():void {
            trace("Content frame3, comp = " + comp);

            if (comp) {
                comp.text = "Level Complete!";
            }
        }
    }
}
