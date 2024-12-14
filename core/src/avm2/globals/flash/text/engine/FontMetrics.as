package flash.text.engine {
    import flash.geom.Rectangle;

    public final class FontMetrics {
        public var emBox:Rectangle;

        public var strikethroughOffset:Number;

        public var strikethroughThickness:Number;

        public var underlineOffset:Number;

        public var underlineThickness:Number;

        public var subscriptOffset:Number;

        public var subscriptScale:Number;

        public var superscriptOffset:Number;

        public var superscriptScale:Number;

        [API("674")]
        public var lineGap:Number;
      
        public function FontMetrics(emBox:Rectangle, strikethroughOffset:Number, strikethroughThickness:Number, underlineOffset:Number, underlineThickness:Number, subscriptOffset:Number, subscriptScale:Number, superscriptOffset:Number, superscriptScale:Number, lineGap:Number = 0) {
            this.emBox = emBox;
            this.strikethroughOffset = strikethroughOffset;
            this.strikethroughThickness = strikethroughThickness;
            this.underlineOffset = underlineOffset;
            this.underlineThickness = underlineThickness;
            this.subscriptOffset = subscriptOffset;
            this.subscriptScale = subscriptScale;
            this.superscriptOffset = superscriptOffset;
            this.superscriptScale = superscriptScale;
            this.lineGap = lineGap;
        }
    }
}

