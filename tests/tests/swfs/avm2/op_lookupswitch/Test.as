package {
    public class Test {
    }
}

function test(s: String) {
	switch(s) {
		case "A":
			trace("A");
			break;
		case "B":
		case "C":
			trace("B/C");
			break;
		default:
			trace("D");
	}
}

test("A");
test("B");
test("C");
test("D");
