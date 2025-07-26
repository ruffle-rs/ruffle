/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

   // TODO: REVIEW AS4 CONVERSION ISSUE



function ToInteger( t ) {
    t = Number( t );

    if ( isNaN( t ) ){
        return ( Number.NaN );
    }
    if ( t == 0 || t == -0 ||
         t == Number.POSITIVE_INFINITY || t == Number.NEGATIVE_INFINITY ) {
         return 0;
    }

    var sign = ( t < 0 ) ? -1 : 1;

    return ( sign * Math.floor( Math.abs( t ) ) );
}




function getTimeZoneDiff()
{
  return -((new Date(2000, 1, 1)).getTimezoneOffset())/60;
}

var msPerDay =          86400000;
var HoursPerDay =       24;
var MinutesPerHour =    60;
var SecondsPerMinute =  60;
var msPerSecond =       1000;
var msPerMinute =       60000;      //  msPerSecond * SecondsPerMinute
var msPerHour =         3600000;    //  msPerMinute * MinutesPerHour
var TZ_DIFF = getTimeZoneDiff();  // offset of tester's timezone from UTC
var             TZ_PST = -8;  // offset of Pacific Standard Time from UTC
var             TZ_IST = +5.5; // offset of Indian Standard Time from UTC
var             IST_DIFF = TZ_DIFF - TZ_IST;  // offset of tester's timezone from IST
var             PST_DIFF = TZ_DIFF - TZ_PST;  // offset of tester's timezone from PST
var TIME_1970    = 0;
var TIME_2000    = 946684800000;
var TIME_1900    = -2208988800000;
var now = new Date();
var TZ_DIFF = getTimeZoneDiff();
var TZ_ADJUST = TZ_DIFF * msPerHour;
var UTC_29_FEB_2000 = TIME_2000 + 31*msPerDay + 28*msPerDay;
var UTC_1_JAN_2005 = TIME_2000 + TimeInYear(2000) + TimeInYear(2001) +
                     TimeInYear(2002) + TimeInYear(2003) + TimeInYear(2004);
var TIME_NOW = now.valueOf();





//  Date test "ResultArrays" are hard-coded for Pacific Standard Time.
//  We must adjust them for the tester's own timezone -
var TIME;
var UTC_YEAR;
var UTC_MONTH;
var UTC_DATE;
var UTC_DAY;
var UTC_HOURS;
var UTC_MINUTES;
var UTC_SECONDS;
var UTC_MS;

var YEAR;
var MONTH;
var DATE;
var DAY;
var HOURS;
var MINUTES;
var SECONDS;
var MS;




function adjustResultArray(ResultArray, msMode)
{
  // If the tester's system clock is in PST, no need to continue -
  if (!PST_DIFF) {return;}

  // The date testcases instantiate Date objects in two different ways:
  //
  //         millisecond mode: e.g.   dt = new Date(10000000);
  //         year-month-day mode:  dt = new Date(2000, 5, 1, ...);
  //
  //  In the first case, the date is measured from Time 0 in Greenwich (i.e. UTC).
  //  In the second case, it is measured with reference to the tester's local timezone.
  //
  //  In the first case we must correct those values expected for local measurements,
  //  like dt.getHours() etc. No correction is necessary for dt.getUTCHours() etc.
  //
  //  In the second case, it is exactly the other way around -

  var t;
  if (msMode)
  {
    // The hard-coded UTC milliseconds from Time 0 derives from a UTC date.
    // Shift to the right by the offset between UTC and the tester.
    t = ResultArray[TIME]  +  TZ_DIFF*msPerHour;

    // Use our date arithmetic functions to determine the local hour, day, etc.
    ResultArray[MINUTES] = MinFromTime(t);
    ResultArray[HOURS] = HourFromTime(t);
    ResultArray[DAY] = WeekDay(t);
    ResultArray[DATE] = DateFromTime(t);
    ResultArray[MONTH] = MonthFromTime(t);
    ResultArray[YEAR] = YearFromTime(t);
  }
  else
  {
    // The hard-coded UTC milliseconds from Time 0 derives from a PST date.
    // Shift to the left by the offset between PST and the tester.
    t = ResultArray[TIME]  -  PST_DIFF*msPerHour;

    // Use our date arithmetic functions to determine the UTC hour, day, etc.
    ResultArray[TIME] = t;
    ResultArray[UTC_MINUTES] = MinFromTime(t);
    ResultArray[UTC_HOURS] = HourFromTime(t);
    ResultArray[UTC_DAY] = WeekDay(t);
    ResultArray[UTC_DATE] = DateFromTime(t);
    ResultArray[UTC_MONTH] = MonthFromTime(t);
    ResultArray[UTC_YEAR] = YearFromTime(t);
  }
}



function Day( t ) {
    return ( Math.floor(t/msPerDay ) );
}
function DaysInYear( y ) {
    if ( y % 4 != 0 ) {
        return 365;
    }
    if ( (y % 4 == 0) && (y % 100 != 0) ) {
        return 366;
    }
    if ( (y % 100 == 0) &&  (y % 400 != 0) ) {
        return 365;
    }
    if ( (y % 400 == 0) ){
        return 366;
    } else {
        _print("ERROR: DaysInYear(" + y + ") case not covered");
        return Number.NaN; //"ERROR: DaysInYear(" + y + ") case not covered";
    }
}
function TimeInYear( y ) {
    return ( DaysInYear(y) * msPerDay );
}
function DayNumber( t ) {
    return ( Math.floor( t / msPerDay ) );
}
function TimeWithinDay( t ) {
    if ( t < 0 ) {
        return ( (t % msPerDay) + msPerDay );
    } else {
        return ( t % msPerDay );
    }
}
function YearNumber( t ) {
}
function TimeFromYear( y ) {
    return ( msPerDay * DayFromYear(y) );
}
function DayFromYear( y ) {
    return (    365*(y-1970) +
                Math.floor((y-1969)/4) -
                Math.floor((y-1901)/100) +
                Math.floor((y-1601)/400) );
}
function InLeapYear( t ) {
    if ( DaysInYear(YearFromTime(t)) == 365 ) {
        return 0;
    }
    if ( DaysInYear(YearFromTime(t)) == 366 ) {
        return 1;
    } else {
        return "ERROR:  InLeapYear("+ t + ") case not covered";
    }
}
function YearFromTime( t ) {
    t = Number( t );
    var sign = ( t < 0 ) ? -1 : 1;
    var year = ( sign < 0 ) ? 1969 : 1970;

    for (   var timeToTimeZero = t; ;  ) {
    //  subtract the current year's time from the time that's left.
        timeToTimeZero -= sign * TimeInYear(year)
        if (isNaN(timeToTimeZero))
            return NaN;

    //  if there's less than the current year's worth of time left, then break.
        if ( sign < 0 ) {
            if ( sign * timeToTimeZero <= 0 ) {
                break;
            } else {
                year += sign;
            }
        } else {
            if ( sign * timeToTimeZero < 0 ) {
                break;
            } else {
                year += sign;
            }
        }
    }
    return ( year );
}
function MonthFromTime( t ) {
    //  i know i could use switch but i'd rather not until it's part of ECMA
    var day = DayWithinYear( t );
    var leap = InLeapYear(t);

    if ( (0 <= day) && (day < 31) ) {
        return 0;
    }
    if ( (31 <= day) && (day < (59+leap)) ) {
        return 1;
    }
    if ( ((59+leap) <= day) && (day < (90+leap)) ) {
        return 2;
    }
    if ( ((90+leap) <= day) && (day < (120+leap)) ) {
        return 3;
    }
    if ( ((120+leap) <= day) && (day < (151+leap)) ) {
        return 4;
    }
    if ( ((151+leap) <= day) && (day < (181+leap)) ) {
        return 5;
    }
    if ( ((181+leap) <= day) && (day < (212+leap)) ) {
        return 6;
    }
    if ( ((212+leap) <= day) && (day < (243+leap)) ) {
        return 7;
    }
    if ( ((243+leap) <= day) && (day < (273+leap)) ) {
        return 8;
    }
    if ( ((273+leap) <= day) && (day < (304+leap)) ) {
        return 9;
    }
    if ( ((304+leap) <= day) && (day < (334+leap)) ) {
        return 10;
    }
    if ( ((334+leap) <= day) && (day < (365+leap)) ) {
        return 11;
    } else {
        return "ERROR:  MonthFromTime("+t+") not known";
    }
}
function DayWithinYear( t ) {
        return( Day(t) - DayFromYear(YearFromTime(t)));
}
function DateFromTime( t ) {
    var day = DayWithinYear(t);
    var month = MonthFromTime(t);
    if ( month == 0 ) {
        return ( day + 1 );
    }
    if ( month == 1 ) {
        return ( day - 30 );
    }
    if ( month == 2 ) {
        return ( day - 58 - InLeapYear(t) );
    }
    if ( month == 3 ) {
        return ( day - 89 - InLeapYear(t));
    }
    if ( month == 4 ) {
        return ( day - 119 - InLeapYear(t));
    }
    if ( month == 5 ) {
        return ( day - 150- InLeapYear(t));
    }
    if ( month == 6 ) {
        return ( day - 180- InLeapYear(t));
    }
    if ( month == 7 ) {
        return ( day - 211- InLeapYear(t));
    }
    if ( month == 8 ) {
        return ( day - 242- InLeapYear(t));
    }
    if ( month == 9 ) {
        return ( day - 272- InLeapYear(t));
    }
    if ( month == 10 ) {
        return ( day - 303- InLeapYear(t));
    }
    if ( month == 11 ) {
        return ( day - 333- InLeapYear(t));
    }

    return ("ERROR:  DateFromTime("+t+") not known" );
}
function WeekDay( t ) {
    var weekday = (Day(t)+4) % 7;
    return( weekday < 0 ? 7 + weekday : weekday );
}

// missing daylight savins time adjustment

function HourFromTime( t ) {
    var h = Math.floor( t / msPerHour ) % HoursPerDay;
    return ( (h<0) ? HoursPerDay + h : h  );
}
function MinFromTime( t ) {
    var min = Math.floor( t / msPerMinute ) % MinutesPerHour;
    return( ( min < 0 ) ? MinutesPerHour + min : min  );
}
function SecFromTime( t ) {
    var sec = Math.floor( t / msPerSecond ) % SecondsPerMinute;
    return ( (sec < 0 ) ? SecondsPerMinute + sec : sec );
}
function msFromTime( t ) {
    var ms = t % msPerSecond;
    return ( (ms < 0 ) ? msPerSecond + ms : ms );
}
function LocalTZA() {
    return ( TZ_DIFF * msPerHour );
}
function UTC( t ) {
    return ( t - LocalTZA() - DaylightSavingTA(t - LocalTZA()) );
}

function DaylightSavingTA( t ) {
    // There is no Daylight saving time in India
    if (IST_DIFF == 0)
        return 0;

    var dst_start;
    var dst_end;
    // Windows fix for 2007 DST change made all previous years follow new DST rules
    // create a date pre-2007 when DST is enabled according to 2007 rules
    var pre_2007:Date = new Date("Mar 20 2006");
    // create a date post-2007
    var post_2007:Date = new Date("Mar 20 2008");
    // if the two dates timezoneoffset match, then this must be a windows box applying
    // post-2007 DST rules to earlier dates.
    var win_machine:Boolean = pre_2007.timezoneOffset == post_2007.timezoneOffset

    if (TZ_DIFF<=-4 && TZ_DIFF>=-8) {
        if (win_machine || YearFromTime(t)>=2007) {
            dst_start = GetSecondSundayInMarch(t) + 2*msPerHour;
            dst_end = GetFirstSundayInNovember(t) + 2*msPerHour;
        } else {
            dst_start = GetFirstSundayInApril(t) + 2*msPerHour;
            dst_end = GetLastSundayInOctober(t) + 2*msPerHour;
        }
    } else {
        dst_start = GetLastSundayInMarch(t) + 2*msPerHour;
        dst_end = GetLastSundayInOctober(t) + 2*msPerHour;
    }
    if ( t >= dst_start && t < dst_end ) {
        return msPerHour;
    } else {
        return 0;

    }

    // Daylight Savings Time starts on the second Sunday    in March at 2:00AM in
    // PST.  Other time zones will need to override this function.
    _print( new Date( UTC(dst_start + LocalTZA())) );

    return UTC(dst_start  + LocalTZA());
}
function GetLastSundayInMarch(t) {
    var year = YearFromTime(t);
    var leap = InLeapYear(t);
    var march = TimeFromYear(year) + TimeInMonth(0,leap) +  TimeInMonth(1,leap)-LocalTZA()+2*msPerHour;
    var sunday;
    for( sunday=march;WeekDay(sunday)>0;sunday +=msPerDay ){;}
    var last_sunday;
    while (true) {
       sunday=sunday+7*msPerDay;
       if (MonthFromTime(sunday)>2)
           break;
       last_sunday=sunday;
    }
    return last_sunday;
}
function GetSecondSundayInMarch(t ) {
    var year = YearFromTime(t);
    var leap = InLeapYear(t);
    var march = TimeFromYear(year) + TimeInMonth(0,leap) +  TimeInMonth(1,leap)-LocalTZA()+2*msPerHour;
    var first_sunday;
    for ( first_sunday = march; WeekDay(first_sunday) >0;
        first_sunday +=msPerDay )
    {
        ;
    }
    var second_sunday=first_sunday+7*msPerDay;
    return second_sunday;
}



function GetFirstSundayInNovember( t ) {
    var year = YearFromTime(t);
    var leap = InLeapYear(t);
        var     nov,m;
    for ( nov = TimeFromYear(year), m = 0; m < 10; m++ )    {
        nov += TimeInMonth(m, leap);
    }
    nov=nov-LocalTZA()+2*msPerHour;
    for ( var first_sunday =    nov; WeekDay(first_sunday)  > 0;
        first_sunday    += msPerDay )
    {
        ;
    }
    return first_sunday;
}
function GetFirstSundayInApril( t ) {
    var year = YearFromTime(t);
    var leap = InLeapYear(t);
    var     apr,m;
    for ( apr = TimeFromYear(year), m = 0; m < 3; m++ ) {
        apr += TimeInMonth(m, leap);
    }
    apr=apr-LocalTZA()+2*msPerHour;

    for ( var first_sunday =    apr; WeekDay(first_sunday)  > 0;
        first_sunday    += msPerDay )
    {
        ;
    }
    return first_sunday;
}
function GetLastSundayInOctober(t) {
    var year = YearFromTime(t);
    var leap = InLeapYear(t);
    var oct,m;
    for (oct =  TimeFromYear(year), m = 0; m < 9; m++ ) {
        oct += TimeInMonth(m, leap);
    }
    oct=oct-LocalTZA()+2*msPerHour;
    var sunday;
    for( sunday=oct;WeekDay(sunday)>0;sunday +=msPerDay ){;}
    var last_sunday;
    while (true) {
       last_sunday=sunday;
       sunday=sunday+7*msPerDay;
       if (MonthFromTime(sunday)>9)
           break;
    }
    return last_sunday;
}


function LocalTime( t ) {
    return ( t + LocalTZA() + DaylightSavingTA(t) );
}
function MakeTime( hour, min, sec, ms ) {
    if ( isNaN( hour ) || isNaN( min ) || isNaN( sec ) || isNaN( ms ) ) {
        return Number.NaN;
    }

    hour = ToInteger(hour);
    min  = ToInteger( min);
    sec  = ToInteger( sec);
    ms   = ToInteger( ms );

    return( (hour*msPerHour) + (min*msPerMinute) +
            (sec*msPerSecond) + ms );
}
function MakeDay( year, month, date ) {
    if ( isNaN(year) || isNaN(month) || isNaN(date) ) {
        return Number.NaN;
    }
    year = ToInteger(year);
    month = ToInteger(month);
    date = ToInteger(date );

    var sign = ( year < 1970 ) ? -1 : 1;
    var t =    ( year < 1970 ) ? 1 :  0;
    var y =    ( year < 1970 ) ? 1969 : 1970;

    var result5 = year + Math.floor( month/12 );
    var result6 = month % 12;

    if ( year < 1970 ) {
       for ( y = 1969; y >= year; y += sign ) {
         t += sign * TimeInYear(y);
       }
    } else {
        for ( y = 1970 ; y < year; y += sign ) {
            t += sign * TimeInYear(y);
        }
    }

    var leap = InLeapYear( t );

    for ( var m = 0; m < month; m++ ) {
        t += TimeInMonth( m, leap );
    }

    if ( YearFromTime(t) != result5 ) {
        return Number.NaN;
    }
    if ( MonthFromTime(t) != result6 ) {
        return Number.NaN;
    }
    if ( DateFromTime(t) != 1 ) {
        return Number.NaN;
    }

    return ( (Day(t)) + date - 1 );
}
function TimeInMonth( month, leap ) {
    // september april june november
    // jan 0  feb 1  mar 2  apr 3   may 4  june 5  jul 6
    // aug 7  sep 8  oct 9  nov 10  dec 11

    if ( month == 3 || month == 5 || month == 8 || month == 10 ) {
        return ( 30*msPerDay );
    }

    // all the rest
    if ( month == 0 || month == 2 || month == 4 || month == 6 ||
         month == 7 || month == 9 || month == 11 ) {
        return ( 31*msPerDay );
     }

    // save february
    return ( (leap == 0) ? 28*msPerDay : 29*msPerDay );
}

function MakeDate( day, time ) {
    if (    day == Number.POSITIVE_INFINITY ||
            day == Number.NEGATIVE_INFINITY ||
            day == Number.NaN ) {
        return Number.NaN;
    }
    if (    time == Number.POSITIVE_INFINITY ||
            time == Number.POSITIVE_INFINITY ||
            day == Number.NaN) {
        return Number.NaN;
    }
    return ( day * msPerDay ) + time;
}


// Compare 2 dates, they are considered equal if the difference is less than 1 second
function compareDate(d1, d2) {
    //Dates may be off by a second
    if (d1 == d2) {
        return true;
    } else if (Math.abs(new Date(d1).getTime() - new Date(d2).getTime()) <= 1000) {
        return true;
    } else {
        return false;
    }
}


function TimeClip( t ) {
    if ( isNaN( t ) ) {
        return ( Number.NaN );
    }
    if ( Math.abs( t ) > 8.64e15 ) {
        return ( Number.NaN );
    }

    return ( ToInteger( t ) );
}


   // TODO: --END-- REVIEW AS4 CONVERSION ISSUE




//     var VERSION = "ECMA_1";
//     var SECTION = "15.9.3.8";
    var TYPEOF  = "object";

    var TIME        = 0;
    var UTC_YEAR    = 1;
    var UTC_MONTH   = 2;
    var UTC_DATE    = 3;
    var UTC_DAY     = 4;
    var UTC_HOURS   = 5;
    var UTC_MINUTES = 6;
    var UTC_SECONDS = 7;
    var UTC_MS      = 8;

    var YEAR        = 9;
    var MONTH       = 10;
    var DATE        = 11;
    var DAY         = 12;
    var HOURS       = 13;
    var MINUTES     = 14;
    var SECONDS     = 15;
    var MS          = 16;


//  for TCMS, the testcases array must be global.
    var tc= 0;
//     var TITLE = "Date constructor:  new Date( value )";
//     var SECTION = "15.9.3.8";
//     var VERSION = "ECMA_1";


    testcases = getTestCases();

//  all tests must call a function that returns a boolean value

function getTestCases( ) {
    var array = new Array();
    var item = 0;

    // all the "ResultArrays" below are hard-coded to Pacific Standard Time values -
    var TZ_ADJUST = -TZ_PST * msPerHour;


    // Dates around 1970
    addNewTestCase( new Date(0),
                    "new Date(0)",
                    [0,1970,0,1,4,0,0,0,0,1969,11,31,3,16,0,0,0] );

    addNewTestCase( new Date(1),
                    "new Date(1)",
                    [1,1970,0,1,4,0,0,0,1,1969,11,31,3,16,0,0,1] );

    addNewTestCase( new Date(true),
                    "new Date(true)",
                    [1,1970,0,1,4,0,0,0,1,1969,11,31,3,16,0,0,1] );

    addNewTestCase( new Date(false),
                    "new Date(false)",
                    [0,1970,0,1,4,0,0,0,0,1969,11,31,3,16,0,0,0] );

    addNewTestCase( new Date( (new Date(0)).toString() ),
                    "new Date(\+ (new Date(0)).toString()+\ )",
                    [0,1970,0,1,4,0,0,0,0,1969,11,31,3,16,0,0,0] );
/*
//    addNewTestCase( "new Date(\""+ (new Date(0)).toLocaleString()+"\")", [0,1970,0,1,4,0,0,0,0,1969,11,31,3,16,0,0,0] );

    addNewTestCase( new Date((new Date(0)).toUTCString()),
                    "new Date(\""+ (new Date(0)).toUTCString()+"\" )",
                    [0,1970,0,1,4,0,0,0,0,1969,11,31,3,16,0,0,0] );

    addNewTestCase( new Date((new Date(1)).toString()),
                    "new Date(\""+ (new Date(1)).toString()+"\" )",
                    [0,1970,0,1,4,0,0,0,0,1969,11,31,3,16,0,0,0] );

    addNewTestCase( new Date( TZ_ADJUST ),
                    "new Date(" + TZ_ADJUST+")",
                    [TZ_ADJUST,1970,0,1,4,8,0,0,0,1970,0,1,4,0,0,0,0] );

    addNewTestCase( new Date((new Date(TZ_ADJUST)).toString()),
                    "new Date(\""+ (new Date(TZ_ADJUST)).toString()+"\")",
                    [TZ_ADJUST,1970,0,1,4,8,0,0,0,1970,0,1,4,0,0,0,0] );

//    addNewTestCase( "new Date(\""+ (new Date(TZ_ADJUST)).toLocaleString()+"\")",[TZ_ADJUST,1970,0,1,4,8,0,0,0,1970,0,1,4,0,0,0,0] );

    addNewTestCase( new Date( (new Date(TZ_ADJUST)).toUTCString() ),
                    "new Date(\""+ (new Date(TZ_ADJUST)).toUTCString()+"\")",
                    [TZ_ADJUST,1970,0,1,4,8,0,0,0,1970,0,1,4,0,0,0,0] );

    // Dates around 2000

    addNewTestCase( new Date(TIME_2000+TZ_ADJUST),
                    "new Date(" +(TIME_2000+TZ_ADJUST)+")",
                    [TIME_2000+TZ_ADJUST,2000,0,1,6,8,0,0,0,2000,0,1,6,0,0,0,0] );

    addNewTestCase( new Date(TIME_2000),
                    "new Date(" +TIME_2000+")",
                    [TIME_2000,2000,0,1,6,0,0,0,0,1999,11,31,5,16,0,0,0] );

    addNewTestCase( new Date( (new Date(TIME_2000+TZ_ADJUST)).toString()),
                    "new Date(\"" +(new Date(TIME_2000+TZ_ADJUST)).toString()+"\")",
                    [TIME_2000+TZ_ADJUST,2000,0,1,6,8,0,0,0,2000,0,1,6,0,0,0,0] );

    addNewTestCase( new Date((new Date(TIME_2000)).toString()),
                   "new Date(\"" +(new Date(TIME_2000)).toString()+"\")",
                   [TIME_2000,2000,0,1,6,0,0,0,0,1999,11,31,5,16,0,0,0] );

//    addNewTestCase( "new Date(\"" +(new Date(TIME_2000+TZ_ADJUST)).toLocaleString()+"\")", [TIME_2000+TZ_ADJUST,2000,0,1,6,8,0,0,0,2000,0,1,6,0,0,0,0] );
//    addNewTestCase( "new Date(\"" +(new Date(TIME_2000)).toLocaleString()+"\")",             [TIME_2000,2000,0,1,6,0,0,0,0,1999,11,31,5,16,0,0,0] );

    addNewTestCase(  new Date( (new Date(TIME_2000+TZ_ADJUST)).toUTCString()),
                    "new Date(\"" +(new Date(TIME_2000+TZ_ADJUST)).toUTCString()+"\")",
                    [TIME_2000+TZ_ADJUST,2000,0,1,6,8,0,0,0,2000,0,1,6,0,0,0,0] );

    addNewTestCase( new Date( (new Date(TIME_2000)).toUTCString()),
                    "new Date(\"" +(new Date(TIME_2000)).toUTCString()+"\")",
                    [TIME_2000,2000,0,1,6,0,0,0,0,1999,11,31,5,16,0,0,0] );

    // Dates around Feb 29, 2000

    var UTC_FEB_29_2000 = TIME_2000 + 31*msPerDay + 28*msPerDay;
    var PST_FEB_29_2000 = UTC_FEB_29_2000 + TZ_ADJUST;

    addNewTestCase( new Date(UTC_FEB_29_2000),
                    "new Date("+UTC_FEB_29_2000+")",
                    [UTC_FEB_29_2000,2000,1,29,2,0,0,0,0,2000,1,28,1,16,0,0,0] );

    addNewTestCase( new Date(PST_FEB_29_2000),
                    "new Date("+PST_FEB_29_2000+")",
                    [PST_FEB_29_2000,2000,1,29,2,8,0,0,0,2000,1,29,2,0,0,0,0] );

    addNewTestCase( new Date( (new Date(UTC_FEB_29_2000)).toString() ),
                    "new Date(\""+(new Date(UTC_FEB_29_2000)).toString()+"\")",
                    [UTC_FEB_29_2000,2000,1,29,2,0,0,0,0,2000,1,28,1,16,0,0,0] );

    addNewTestCase( new Date( (new Date(PST_FEB_29_2000)).toString() ),
                    "new Date(\""+(new Date(PST_FEB_29_2000)).toString()+"\")",
                    [PST_FEB_29_2000,2000,1,29,2,8,0,0,0,2000,1,29,2,0,0,0,0] );

//  Parsing toLocaleString() is not guaranteed by ECMA.
//    addNewTestCase( "new Date(\""+(new Date(UTC_FEB_29_2000)).toLocaleString()+"\")",  [UTC_FEB_29_2000,2000,1,29,2,0,0,0,0,2000,1,28,1,16,0,0,0] );
//    addNewTestCase( "new Date(\""+(new Date(PST_FEB_29_2000)).toLocaleString()+"\")",  [PST_FEB_29_2000,2000,1,29,2,8,0,0,0,2000,1,29,2,0,0,0,0] );

// 28th October 2004
//  Removing the occurance of the toGMTString from this file.
//  As toGMTString will not be supported by ActionScript 3.0
//  in the current state.

 //   addNewTestCase( new Date( (new Date(UTC_FEB_29_2000)).toGMTString() ),
 //                   "new Date(\""+(new Date(UTC_FEB_29_2000)).toGMTString()+"\")",
 //                   [UTC_FEB_29_2000,2000,1,29,2,0,0,0,0,2000,1,28,1,16,0,0,0] );

 //   addNewTestCase( new Date( (new Date(PST_FEB_29_2000)).toGMTString() ),
 //                   "new Date(\""+(new Date(PST_FEB_29_2000)).toGMTString()+"\")",
 //                   [PST_FEB_29_2000,2000,1,29,2,8,0,0,0,2000,1,29,2,0,0,0,0] );

    // Dates around 1900

    var PST_1900 = TIME_1900 + 8*msPerHour;

    addNewTestCase( new Date( TIME_1900 ),
                    "new Date("+TIME_1900+")",
                    [TIME_1900,1900,0,1,1,0,0,0,0,1899,11,31,0,16,0,0,0] );

    addNewTestCase( new Date(PST_1900),
                    "new Date("+PST_1900+")",
                    [ PST_1900,1900,0,1,1,8,0,0,0,1900,0,1,1,0,0,0,0] );

    addNewTestCase( new Date( (new Date(TIME_1900)).toString() ),
                    "new Date(\""+(new Date(TIME_1900)).toString()+"\")",
                    [TIME_1900,1900,0,1,1,0,0,0,0,1899,11,31,0,16,0,0,0] );

    addNewTestCase( new Date( (new Date(PST_1900)).toString() ),
                    "new Date(\""+(new Date(PST_1900 )).toString()+"\")",
                    [ PST_1900,1900,0,1,1,8,0,0,0,1900,0,1,1,0,0,0,0] );

    addNewTestCase( new Date( (new Date(TIME_1900)).toUTCString() ),
                    "new Date(\""+(new Date(TIME_1900)).toUTCString()+"\")",
                    [TIME_1900,1900,0,1,1,0,0,0,0,1899,11,31,0,16,0,0,0] );

    addNewTestCase( new Date( (new Date(PST_1900)).toUTCString() ),
                    "new Date(\""+(new Date(PST_1900 )).toUTCString()+"\")",
                    [ PST_1900,1900,0,1,1,8,0,0,0,1900,0,1,1,0,0,0,0] );

//    addNewTestCase( "new Date(\""+(new Date(TIME_1900)).toLocaleString()+"\")",   [TIME_1900,1900,0,1,1,0,0,0,0,1899,11,31,0,16,0,0,0] );
//    addNewTestCase( "new Date(\""+(new Date(PST_1900 )).toLocaleString()+"\")",   [ PST_1900,1900,0,1,1,8,0,0,0,1900,0,1,1,0,0,0,0] );


//   This test case is incorrect.  Need to fix the DaylightSavings functions in
//   shell.js for this to work properly.

    var DST_START_1998 = UTC( GetFirstSundayInApril(TimeFromYear(1998)) + 2*msPerHour )

    addNewTestCase( new Date(DST_START_1998-1),
                    "new Date("+(DST_START_1998-1)+")",
                    [DST_START_1998-1,1998,3,5,0,9,59,59,999,1998,3,5,0,1,59,59,999] );

    addNewTestCase( new Date(DST_START_1998),
                    "new Date("+DST_START_1998+")",
                    [DST_START_1998,1998,3,5,0,10,0,0,0,1998,3,5,0,3,0,0,0]);

    var DST_END_1998 = UTC( GetLastSundayInOctober(TimeFromYear(1998)) + 2*msPerHour );

    addNewTestCase ( new Date(DST_END_1998-1),
                    "new Date("+(DST_END_1998-1)+")",
                    [DST_END_1998-1,1998,9,25,0,8,59,59,999,1998,9,25,0,1,59,59,999] );

    addNewTestCase ( new Date(DST_END_1998),
                    "new Date("+DST_END_1998+")",
                    [DST_END_1998,1998,9,25,0,9,0,0,0,1998,9,25,0,1,0,0,0] );
*/

    function addNewTestCase( DateCase, DateString, ResultArray ) {
        //adjust hard-coded ResultArray for tester's timezone instead of PST
        adjustResultArray(ResultArray, 'msMode');

        item = array.length;

        array[item++] = Assert.expectEq(  DateString+".getTime()", ResultArray[TIME],       DateCase.getTime() );
        array[item++] = Assert.expectEq(  DateString+".valueOf()", ResultArray[TIME],       DateCase.valueOf() );
        array[item++] = Assert.expectEq(  DateString+".getUTCFullYear()",      ResultArray[UTC_YEAR], DateCase.getUTCFullYear() );
        array[item++] = Assert.expectEq(  DateString+".getUTCMonth()",         ResultArray[UTC_MONTH],  DateCase.getUTCMonth() );
        array[item++] = Assert.expectEq(  DateString+".getUTCDate()",          ResultArray[UTC_DATE],   DateCase.getUTCDate() );
        array[item++] = Assert.expectEq(  DateString+".getUTCDay()",           ResultArray[UTC_DAY],    DateCase.getUTCDay() );
        array[item++] = Assert.expectEq(  DateString+".getUTCHours()",         ResultArray[UTC_HOURS],  DateCase.getUTCHours() );
        array[item++] = Assert.expectEq(  DateString+".getUTCMinutes()",       ResultArray[UTC_MINUTES],DateCase.getUTCMinutes() );
        array[item++] = Assert.expectEq(  DateString+".getUTCSeconds()",       ResultArray[UTC_SECONDS],DateCase.getUTCSeconds() );
        array[item++] = Assert.expectEq(  DateString+".getUTCMilliseconds()",  ResultArray[UTC_MS],     DateCase.getUTCMilliseconds() );
        array[item++] = Assert.expectEq(  DateString+".getFullYear()",         ResultArray[YEAR],       DateCase.getFullYear() );
        array[item++] = Assert.expectEq(  DateString+".getMonth()",            ResultArray[MONTH],      DateCase.getMonth() );
        array[item++] = Assert.expectEq(  DateString+".getDate()",             ResultArray[DATE],       DateCase.getDate() );
        array[item++] = Assert.expectEq(  DateString+".getDay()",              ResultArray[DAY],        DateCase.getDay() );
        array[item++] = Assert.expectEq(  DateString+".getHours()",            ResultArray[HOURS],      DateCase.getHours() );
        array[item++] = Assert.expectEq(  DateString+".getMinutes()",          ResultArray[MINUTES],    DateCase.getMinutes() );
        array[item++] = Assert.expectEq(  DateString+".getSeconds()",          ResultArray[SECONDS],    DateCase.getSeconds() );
        array[item++] = Assert.expectEq(  DateString+".getMilliseconds()",     ResultArray[MS],         DateCase.getMilliseconds() );
    }

    return ( array );
}
