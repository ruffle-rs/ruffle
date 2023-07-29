/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "String/replace-001.as";
//     var VERSION = "ECMA_2";
//     var TITLE   = "String.prototype.replace( regexp, replaceValue )";


    /*
     * If regexp is not an object of type RegExp, it is replaced with the
     * result of the expression new RegExp(regexp).  Let string denote the
     * result of converting the this value to a string.  String is searched
     * for the first occurrence of the regular expression pattern regexp if
     * regexp.global is false, or all occurrences if regexp.global is true.
     *
     * The match is performed as in String.prototype.match, including the
     * update of regexp.lastIndex.  Let m be the number of matched
     * parenthesized subexpressions as specified in section 15.7.5.3.
     *
     * If replaceValue is a function, then for each matched substring, call
     * the function with the following m + 3 arguments. Argument 1 is the
     * substring that matched. The next m arguments are all of the matched
     * subexpressions. Argument m + 2 is the length of the left context, and
     * argument m + 3 is string.
     *
     * The result is a string value derived from the original input by
     * replacing each matched substring with the corresponding return value
     * of the function call, converted to a string if need be.
     *
     * Otherwise, let newstring denote the result of converting replaceValue
     * to a string. The result is a string value derived from the original
     * input string by replacing each matched substring with a string derived
     * from newstring by replacing characters in newstring by replacement text
     * as specified in the following table:
     *
     * $& The matched substring.
     * $‘ The portion of string that precedes the matched substring.
     * $’ The portion of string that follows the matched substring.
     * $+ The substring matched by the last parenthesized subexpressions in
     *      the regular expression.
     * $n The corresponding matched parenthesized subexpression n, where n
     * is a single digit 0-9. If there are fewer than n subexpressions, “$n
     * is left unchanged.
     *
     * Note that the replace function is intentionally generic; it does not
     * require that its this value be a string object. Therefore, it can be
     * transferred to other kinds of objects for use as a method.
     */


	


//-----------------------------------------------------------------------------

//     var SECTION = "eregress_104375";
//     var VERSION = "";

//     var TITLE   = "Testing String.prototype.replace( regexp, replaceValue )";


    var testcases = getTestCases();

function inSection(x) {
   return "Section "+x+" of test -";
}

function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var UBound = 0;
    var summary="";
    var status = '';
    var statusitems = [];
    var actual = '';
    var actualvalues = [];
    var expect= '';
    var expectedvalues = [];


    /*
     * Use the regexp to replace 'uid=31' with 'uid=15'
     *
     * In the second parameter of string.replace() method,
     * "$1" refers to the first backreference: 'uid='
     */
    var str = 'She sells seashells by the seashore.';
    var re = /sh/g;


    status = inSection(1);
    actual  = str.replace (re,'sch');
    expect = 'She sells seaschells by the seaschore.';
    array[item++] = Assert.expectEq( status, expect, actual);


    status = inSection(2);
    actual  = str.replace (re, "$$" + 'sch');
    expect = 'She sells sea$schells by the sea$schore.';
    array[item++] = Assert.expectEq( status, expect, actual);


    status = inSection(3);
    actual  = str.replace (re, "$&" + 'sch');
    expect = 'She sells seashschells by the seashschore.';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = inSection(4);
    actual  = str.replace (re, "$`" + 'sch');
    expect = 'She sells seaShe sells seaschells by the seaShe sells seashells by the seaschore.';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = inSection(5);
    actual  = str.replace (re, "$'" + 'sch');
    expect = 'She sells seaells by the seashore.schells by the seaore.schore.';
    array[item++] = Assert.expectEq( status, expect, actual);

    var str2 = 'She sells seashells by the seashore.';
    var re = /sh/;


    status = inSection(6);
    actual  = str2.replace (re,'sch');
    expect = 'She sells seaschells by the seashore.';
    array[item++] = Assert.expectEq( status, expect, actual);


    status = inSection(7);
    actual  = str2.replace (re, "$$" + 'sch');
    expect = 'She sells sea$schells by the seashore.';
    array[item++] = Assert.expectEq( status, expect, actual);


    status = inSection(8);
    actual  = str2.replace (re, "$&" + 'sch');
    expect = 'She sells seashschells by the seashore.';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = inSection(9);
    actual  = str2.replace (re, "$`" + 'sch');
    expect = 'She sells seaShe sells seaschells by the seashore.';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = inSection(10);
    actual  = str2.replace (re, "$'" + 'sch');
    expect = 'She sells seaells by the seashore.schells by the seashore.';
    array[item++] = Assert.expectEq( status, expect, actual);

    var str1 = 'uid=31';
    var re1 = /(uid=)(\d+)/;

    status = inSection(11);
    actual  = str1.replace (re1, "$11" + 15);
    expect = 'uid=115';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = inSection(12);
    actual  = str1.replace (re1, "$11" + '15');
    expect = 'uid=115';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = inSection(13);
    actual  = str1.replace (re1, "$11" + 'A15');
    expect = 'uid=1A15';
    array[item++] = Assert.expectEq( status, expect, actual);

    var str1:String = "abc12 def34";
    var pattern:RegExp = /([a-z]+)([0-9]+)/;
    var str2:String = str1.replace(pattern, replFN);
 
    function replFN():String {
        return arguments[2] + arguments[1];
    }

    status = inSection(14);
    actual  = str2
    expect = '12abc def34';
    array[item++] = Assert.expectEq( status, expect, actual);

    var str1:String = "abc12 def34";
    var pattern:RegExp = /([a-z]+)([0-9]+)/g;
    var str2:String = str1.replace(pattern, replFN);
 
    function replFN():String {
        return arguments[2] + arguments[1];
    }

    status = inSection(14);
    actual  = str2
    expect = '12abc 34def';
    array[item++] = Assert.expectEq( status, expect, actual);

    return array;
}




