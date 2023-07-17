package flash.globalization {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_method;

    public final class CurrencyFormatter {

        public var fractionalDigits: int = 0;

        public function CurrencyFormatter(locale: String) {
            stub_constructor("flash.globalization.CurrencyFormatter");
        }

        public function format(value:Number, withCurrencySymbol:Boolean = false):String {
            stub_method("flash.globalization.CurrencyFormatter", "format");
            return value.toString();
        }
    }
}
