class Test {
    static function main() {
		trace("\"A\" < \"B\"");
		trace("A" < "B");
		trace("\"\\uFF61\" < \"\\uD800\\uDC02\"");
		trace("\uFF61" < "\uD800\uDC02");
	}
}
