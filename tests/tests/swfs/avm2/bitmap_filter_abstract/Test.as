package {
    import flash.display.Sprite;
    import flash.filters.BitmapFilter;

    public class Test extends Sprite {
        public function Test() {
            try {
                new BitmapFilter();
                trace("Constructed");
            } catch (e) {
                trace("Caught: " + e.getStackTrace());
            }
            try {
                new CustomFilter();
                trace("Constructed");
            } catch (e) {
                trace("Caught: " + e.getStackTrace());
            }
            try {
                new CustomConvolutionFilter();
                trace("Constructed");
            } catch (e) {
                trace("Caught: " + e.getStackTrace());
            }

            this.filters = [new CustomConvolutionFilter()];
            trace("CustomConvolutionFilter set");
        }
    }
}

import flash.filters.BitmapFilter;
import flash.filters.ConvolutionFilter;

class CustomFilter extends BitmapFilter {
    public function CustomFilter() {
        super();
    }
}

class CustomConvolutionFilter extends ConvolutionFilter {
    public function CustomConvolutionFilter() {
        super();
    }
}
