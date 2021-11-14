package {
	public class Test {
	}
}

function assert_exp(val) {
	trace("///(radix unspecified)");
	trace(val.toString());
	trace("///(radix = 2)");
	trace(val.toString(2));
	trace("///(radix = 3)");
	trace(val.toString(3));
	trace("///(radix = 4)");
	trace(val.toString(4));
	trace("///(radix = 5)");
	trace(val.toString(5));
	trace("///(radix = 6)");
	trace(val.toString(6));
	trace("///(radix = 7)");
	trace(val.toString(7));
	trace("///(radix = 8)");
	trace(val.toString(8));
	trace("///(radix = 9)");
	trace(val.toString(9));
	trace("///(radix = 10)");
	trace(val.toString(10));
	trace("///(radix = 11)");
	trace(val.toString(11));
	trace("///(radix = 12)");
	trace(val.toString(12));
	trace("///(radix = 13)");
	trace(val.toString(13));
	trace("///(radix = 14)");
	trace(val.toString(14));
	trace("///(radix = 15)");
	trace(val.toString(15));
	trace("///(radix = 16)");
	trace(val.toString(16));
	trace("///(radix = 17)");
	trace(val.toString(17));
	trace("///(radix = 18)");
	trace(val.toString(18));
	trace("///(radix = 19)");
	trace(val.toString(19));
	trace("///(radix = 20)");
	trace(val.toString(20));
	trace("///(radix = 21)");
	trace(val.toString(21));
	trace("///(radix = 22)");
	trace(val.toString(22));
	trace("///(radix = 23)");
	trace(val.toString(23));
	trace("///(radix = 24)");
	trace(val.toString(24));
	trace("///(radix = 25)");
	trace(val.toString(25));
	trace("///(radix = 26)");
	trace(val.toString(26));
	trace("///(radix = 27)");
	trace(val.toString(27));
	trace("///(radix = 28)");
	trace(val.toString(28));
	trace("///(radix = 29)");
	trace(val.toString(29));
	trace("///(radix = 30)");
	trace(val.toString(30));
	trace("///(radix = 31)");
	trace(val.toString(31));
	trace("///(radix = 32)");
	trace(val.toString(32));
	trace("///(radix = 33)");
	trace(val.toString(33));
	trace("///(radix = 34)");
	trace(val.toString(34));
	trace("///(radix = 35)");
	trace(val.toString(35));
	trace("///(radix = 36)");
	trace(val.toString(36));
	trace("///(valueOf)");
	trace(val.valueOf());
}

trace("//1.2315e-8");
assert_exp(1.2315e-8);

trace("//1.2315e-7");
assert_exp(1.2315e-7);

trace("//1.2315e-6");
assert_exp(1.2315e-6);

trace("//1.2315e2");
assert_exp(1.2315e2);

trace("//1.2315e19");
assert_exp(1.2315e19);

trace("//1.2315e20");
assert_exp(1.2315e20);

trace("//1.2315e21");
assert_exp(1.2315e21);

trace("//1.2315987654321987654321987654321987654321987654321987654321e-8");
assert_exp(1.2315987654321987654321987654321987654321987654321987654321e-8);

trace("//1.2315987654321987654321987654321987654321987654321987654321e-7");
assert_exp(1.2315987654321987654321987654321987654321987654321987654321e-7);

trace("//1.2315987654321987654321987654321987654321987654321987654321e-6");
assert_exp(1.2315987654321987654321987654321987654321987654321987654321e-6);

trace("//1.2315987654321987654321987654321987654321987654321987654321e2");
assert_exp(1.2315987654321987654321987654321987654321987654321987654321e2);

trace("//1.2315987654321987654321987654321987654321987654321987654321e19");
assert_exp(1.2315987654321987654321987654321987654321987654321987654321e19);

trace("//1.2315987654321987654321987654321987654321987654321987654321e20");
assert_exp(1.2315987654321987654321987654321987654321987654321987654321e20);

trace("//1.2315987654321987654321987654321987654321987654321987654321e21");
assert_exp(1.2315987654321987654321987654321987654321987654321987654321e21);