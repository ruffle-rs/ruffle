package {
	public class Test {
	}
}

function testParser(date, useUtc) {
	trace("TESTING", date);
	var milliseconds = Date.parse(date);
	if (isNaN(milliseconds)) {
		trace("Failed.");
		return;
	}
	var date = new Date(milliseconds);
	if (useUtc) {
		trace(date.fullYearUTC, date.monthUTC, date.dateUTC, date.dayUTC, date.hoursUTC, date.minutesUTC, date.secondsUTC, date.millisecondsUTC);
	} else {
		trace(date.fullYear, date.month, date.date, date.day, date.hours, date.minutes, date.seconds, date.milliseconds);
	}
}
trace("// TEST VALID DATES")
testParser("Wed Apr 12 15:30:17 2006 GMT-0700", true);
testParser("Wed Apr 12 15:30:17 2006 GMT+0700", true);
testParser("Wed Apr 12 15:30:17 2006 GMT-0200", true);
testParser("Sat Apr 30 1974", false);
testParser("1999 Mon Sun Sat Apr 30", false);
testParser("1999 Mon Sun Sat Apr 30 15:30:17", false);
testParser("Apr/03/1988 15:30:17", false);
testParser("15:30:17    Apr/03/1988   ", false);
testParser("Sat Apr 30 77", false);

trace("// TEST INVALID DATES");
testParser("Wed Apr 12 15:30:17 GMT-0700");
testParser("Wed 12 15:30:17 GMT-0700 2006");
testParser("Sat Jan 30");
testParser("Sat Jan 70 77");
testParser("Sat Jan 30 random 77");
testParser("Sat Jan Oct 30 77");
testParser("Sat Jan 30 77 Apr/03/1988");
testParser("Sat Jan 30 77 GMT-0700 GMT-0800");
