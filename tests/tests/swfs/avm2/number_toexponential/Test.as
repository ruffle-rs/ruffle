package {
	public class Test {
	}
}

function assert_exp(val) {
	trace("///(digits = ?!)");
	trace(val.toExponential());
	trace("///(digits = 0)");
	trace(val.toExponential(0));
	trace("///(digits = 1)");
	trace(val.toExponential(1));
	trace("///(digits = 2)");
	trace(val.toExponential(2));
	trace("///(digits = 3)");
	trace(val.toExponential(3));
	trace("///(digits = 4)");
	trace(val.toExponential(4));
	trace("///(digits = 5)");
	trace(val.toExponential(5));
	trace("///(digits = 6)");
	trace(val.toExponential(6));
	trace("///(digits = 7)");
	trace(val.toExponential(7));
	trace("///(digits = 8)");
	trace(val.toExponential(8));
	trace("///(digits = 9)");
	trace(val.toExponential(9));
	trace("///(digits = 10)");
	trace(val.toExponential(10));
	trace("///(digits = 20)");
	trace(val.toExponential(20));
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