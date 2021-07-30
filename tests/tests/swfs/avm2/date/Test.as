package {
	public class Test {
	}
}

trace("// Date with specific time stamp");
var date = new Date(929156400000);
trace("// Date(929156400000)");
trace(date.fullYearUTC, date.monthUTC, date.dateUTC, date.dayUTC, date.hoursUTC, date.minutesUTC, date.secondsUTC);

trace("// Date with fields chosen");
var date = new Date(2021, 7, 29, 4, 22, 55, 11);
trace("// Date(2021, 7, 29, 4, 22, 55, 11)")
trace(date.fullYear, date.month, date.date, date.day, date.hours, date.minutes, date.seconds);

trace("// Setting the date after construction");
date.fullYear = 1999;
date.month = 2;
date.date = 31;
trace(date.fullYear, date.month, date.date);

trace("// Setting the date after construction using setter methods");
date.setFullYear(1988, 5, 2);

trace("// Using getter methods")
trace(date.getFullYear(), date.getMonth(), date.getDate(), date.getHours(), date.getMinutes(), date.getSeconds(), date.getMilliseconds());


var date = new Date();
date.setUTCFullYear(1999, 5, 3);
date.setUTCHours(9, 8, 5, 3)
trace(date.fullYearUTC, date.monthUTC, date.dateUTC, date.dayUTC, date.hoursUTC, date.minutesUTC, date.secondsUTC, date.millisecondsUTC);

trace("// Using getter methods")
trace(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate(), date.getUTCHours(), date.getUTCMinutes(), date.getUTCSeconds(), date.getUTCMilliseconds());
